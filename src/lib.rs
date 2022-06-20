use neon::{prelude::*, types::buffer::TypedArray};
use std::collections::HashMap;
use std::{ops::Deref, sync::Arc};
use wry::{
    application::{
        dpi::{PhysicalPosition, PhysicalSize, Size},
        event::{Event, StartCause, WindowEvent},
        event_loop::{ControlFlow, EventLoop, EventLoopProxy, EventLoopWindowTarget},
        platform::windows::EventLoopExtWindows,
        window::{Icon, Window, WindowBuilder, WindowId},
    },
    webview::{WebContext, WebView, WebViewBuilder},
};

enum UserEvents {
    UnsafeQuit(Root<JsFunction>),
    CreateNewWindow(Options, Root<JsFunction>),
    CloseWindow(WindowId, Root<JsFunction>),
    CenterWindow(WindowId, Root<JsFunction>),
    ChangeTitleWindow(WindowId, String, Root<JsFunction>),
    SetVisibleWindow(WindowId, bool, Root<JsFunction>),
    SetResizableWindow(WindowId, bool, Root<JsFunction>),
    EvaluateScript(WindowId, String, Root<JsFunction>),
    SetWindowSize(WindowId, u32, u32, Root<JsFunction>),
    GetWindowSize(WindowId, Root<JsFunction>),
    SetMinimizedWindow(WindowId, bool, Root<JsFunction>),
    DragWindow(WindowId),
    FocusWindow(WindowId, Root<JsFunction>),
    SetAlwaysOnTopWindow(WindowId, bool, Root<JsFunction>),
    SetIgnoreCursorEvents(WindowId, bool, Root<JsFunction>),
    OpenDevtools(WindowId, Root<JsFunction>),
    CloseDevtools(WindowId, Root<JsFunction>),
    SetFramelessWindow(WindowId, bool, Root<JsFunction>),
    SetWindowIcon(WindowId, Vec<u8>, u32, u32, Root<JsFunction>),
    IpcPostMessage(WindowId, String),
}

struct Options {
    title: String,
    devtools: bool,
    transparent: bool,
    frameless: bool,
    width: u32,
    height: u32,
    visible: bool,
    resizable: bool,
    initialization_script: String,
}

struct IpcBoxed {
    proxy: Arc<EventLoopProxy<UserEvents>>,
}
impl Finalize for IpcBoxed {}

struct WindowIdBoxed {
    window_id: WindowId,
}
impl Finalize for WindowIdBoxed {}

fn resolve_node_promise(channel: Channel, cb: Root<JsFunction>) {
    let _ = channel
        .send(move |mut cx| {
            let this = cx.undefined();
            let callback = cb.into_inner(&mut cx);
            let _ = callback.call(&mut cx, this, &[]);
            Ok(())
        })
        .join();
}

fn create_new_window(
    options: Options,
    event_loop: &EventLoopWindowTarget<UserEvents>,
    proxy: EventLoopProxy<UserEvents>,
) -> (WindowId, WebView) {
    let window = WindowBuilder::new()
        .with_title(options.title)
        .with_inner_size(Size::new(PhysicalSize::new(options.width, options.height)))
        .with_visible(options.visible)
        .with_resizable(options.resizable)
        .with_transparent(options.transparent)
        .with_decorations(!options.frameless)
        .build(event_loop)
        .unwrap();
    let window_id = window.id();

    let handler = move |window: &Window, req: String| match req.as_str() {
        "drag-window" => {
            let _ = proxy.send_event(UserEvents::DragWindow(window.id()));
        }
        _ if req.starts_with("ipc:") => {
            let message = req.replace("ipc:", "");
            let _ = proxy.send_event(UserEvents::IpcPostMessage(window.id(), message));
        }
        _ => {}
    };

    let data_directory = std::env::temp_dir();
    let mut web_context = WebContext::new(Some(data_directory));
    let webview = WebViewBuilder::new(window)
        .unwrap()
        // always should be fill
        .with_initialization_script(&options.initialization_script)
        .with_transparent(options.transparent)
        .with_devtools(options.devtools)
        .with_web_context(&mut web_context)
        .with_ipc_handler(handler)
        .with_html("")
        .unwrap()
        .build()
        .unwrap();
    (window_id, webview)
}

fn app_init(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let listener_cb = cx.argument::<JsFunction>(0)?.root(&mut cx);
    let result_cb = cx.argument::<JsFunction>(1)?.root(&mut cx);

    let listener_cb = Arc::new(listener_cb);
    let result_cb = Arc::new(result_cb);
    let channel = cx.channel();
    std::thread::spawn(move || {
        let event_loop: EventLoop<UserEvents> = EventLoop::new_any_thread();
        let proxy = event_loop.create_proxy();
        let mut webviews = HashMap::new();
        std::panic::set_hook(Box::new(move |panic_info| {
            println!("{}", panic_info);
        }));
        event_loop.run(move |event, event_loop, control_flow| {
            *control_flow = ControlFlow::Wait;
            let listener_cb = listener_cb.clone();
            let result_cb = result_cb.clone();
            let proxy = proxy.clone();
            match event {
                Event::NewEvents(StartCause::Init) => {
                    channel.send(move |mut cx| {
                        let this = cx.undefined();
                        let callback = result_cb.to_inner(&mut cx);
                        let proxy_boxed = cx.boxed(IpcBoxed {
                            proxy: Arc::new(proxy),
                        });
                        let _ = callback.call(&mut cx, this, &[proxy_boxed.upcast()]);
                        Ok(())
                    });
                }
                Event::UserEvent(UserEvents::CreateNewWindow(option, cb)) => {
                    let (window_id, webview) =
                        create_new_window(option, &event_loop, proxy.clone());
                    webviews.insert(window_id, webview);

                    channel.send(move |mut cx| {
                        let this = cx.undefined();
                        let callback = cb.into_inner(&mut cx);
                        let window_id_boxed = cx.boxed(WindowIdBoxed { window_id });
                        let _ = callback.call(&mut cx, this, &[window_id_boxed.upcast()]);
                        Ok(())
                    });
                }
                Event::UserEvent(UserEvents::CloseWindow(window_id, cb)) => {
                    resolve_node_promise(channel.clone(), cb);
                    webviews.remove(&window_id);
                }
                Event::UserEvent(UserEvents::CenterWindow(window_id, cb)) => {
                    let webview = webviews.get(&window_id).unwrap();
                    let window = webview.window();
                    if let Some(monitor) = window.current_monitor() {
                        let screen_size = monitor.size();
                        let window_size = window.inner_size();
                        let x = (screen_size.width - window_size.width) / 2;
                        let y = (screen_size.height - window_size.height) / 2;
                        window.set_outer_position(PhysicalPosition::new(x, y));
                        resolve_node_promise(channel.clone(), cb);
                    } else {
                        panic!("Could not get current monitor");
                    }
                }
                Event::UserEvent(UserEvents::ChangeTitleWindow(window_id, title, cb)) => {
                    let webview = webviews.get(&window_id).unwrap();
                    let window = webview.window();
                    window.set_title(&title);
                    resolve_node_promise(channel.clone(), cb);
                }
                Event::UserEvent(UserEvents::SetVisibleWindow(window_id, visible, cb)) => {
                    let webview = webviews.get(&window_id).unwrap();
                    let window = webview.window();
                    window.set_visible(visible);
                    resolve_node_promise(channel.clone(), cb);
                }
                Event::UserEvent(UserEvents::SetResizableWindow(window_id, resizable, cb)) => {
                    let webview = webviews.get(&window_id).unwrap();
                    let window = webview.window();
                    window.set_resizable(resizable);
                    resolve_node_promise(channel.clone(), cb);
                }
                Event::UserEvent(UserEvents::EvaluateScript(window_id, script, cb)) => {
                    let webview = webviews.get(&window_id).unwrap();
                    let _ = webview.evaluate_script(&script);
                    resolve_node_promise(channel.clone(), cb);
                }
                Event::UserEvent(UserEvents::SetWindowSize(window_id, width, height, cb)) => {
                    let webview = webviews.get(&window_id).unwrap();
                    let window = webview.window();
                    window.set_inner_size(Size::new(PhysicalSize::new(width, height)));
                    resolve_node_promise(channel.clone(), cb);
                }
                Event::UserEvent(UserEvents::GetWindowSize(window_id, cb)) => {
                    let webview = webviews.get(&window_id).unwrap();
                    let window = webview.window();
                    let size = window.inner_size();
                    channel.send(move |mut cx| {
                        let this = cx.undefined();
                        let callback = cb.into_inner(&mut cx);
                        let width = cx.number(size.width as f64);
                        let height = cx.number(size.height as f64);
                        let _ = callback.call(&mut cx, this, &[width.upcast(), height.upcast()]);
                        Ok(())
                    });
                }
                Event::UserEvent(UserEvents::SetMinimizedWindow(window_id, minimized, cb)) => {
                    let webview = webviews.get(&window_id).unwrap();
                    let window = webview.window();
                    window.set_minimized(minimized);
                    resolve_node_promise(channel.clone(), cb);
                }
                Event::UserEvent(UserEvents::FocusWindow(window_id, cb)) => {
                    let webview = webviews.get(&window_id).unwrap();
                    let window = webview.window();
                    window.set_focus();
                    resolve_node_promise(channel.clone(), cb);
                }
                Event::UserEvent(UserEvents::SetAlwaysOnTopWindow(window_id, flag, cb)) => {
                    let webview = webviews.get(&window_id).unwrap();
                    let window = webview.window();
                    window.set_always_on_top(flag);
                    resolve_node_promise(channel.clone(), cb);
                }
                Event::UserEvent(UserEvents::SetIgnoreCursorEvents(window_id, flag, cb)) => {
                    let webview = webviews.get(&window_id).unwrap();
                    let window = webview.window();
                    if let Ok(()) = window.set_ignore_cursor_events(flag) {
                        resolve_node_promise(channel.clone(), cb);
                    } else {
                        panic!("Is not available on this platform(mobile)");
                    }
                }
                Event::UserEvent(UserEvents::OpenDevtools(window_id, cb)) => {
                    let webview = webviews.get(&window_id).unwrap();
                    webview.open_devtools();
                    resolve_node_promise(channel.clone(), cb);
                }
                Event::UserEvent(UserEvents::CloseDevtools(window_id, cb)) => {
                    let webview = webviews.get(&window_id).unwrap();
                    webview.close_devtools();
                    resolve_node_promise(channel.clone(), cb);
                }
                Event::UserEvent(UserEvents::SetFramelessWindow(window_id, flag, cb)) => {
                    let webview = webviews.get(&window_id).unwrap();
                    let window = webview.window();
                    window.set_decorations(!flag);
                    resolve_node_promise(channel.clone(), cb);
                }
                Event::UserEvent(UserEvents::SetWindowIcon(window_id, rgba, width, height, cb)) => {
                    let icon = Icon::from_rgba(rgba, width, height);
                    match icon {
                        Ok(icon) => {
                            let webview = webviews.get(&window_id).unwrap();
                            let window = webview.window();
                            window.set_window_icon(Some(icon));
                            resolve_node_promise(channel.clone(), cb);
                        }
                        Err(err) => {
                            panic!("{}", err);
                        }
                    }
                }

                Event::UserEvent(UserEvents::UnsafeQuit(cb)) => {
                    resolve_node_promise(channel.clone(), cb);
                    panic!("unsafe quit");
                }
                Event::UserEvent(UserEvents::DragWindow(window_id)) => {
                    let webview = webviews.get(&window_id).unwrap();
                    let window = webview.window();
                    let _ = window.drag_window();
                }
                Event::UserEvent(UserEvents::IpcPostMessage(window_id, message)) => {
                    channel.send(move |mut cx| {
                        let this = cx.undefined();
                        let callback = listener_cb.to_inner(&mut cx);
                        let event = cx.string("ipc");
                        let window_id_boxed = cx.boxed(WindowIdBoxed { window_id });
                        let message = cx.string(message);
                        let _ = callback.call(
                            &mut cx,
                            this,
                            &[event.upcast(), window_id_boxed.upcast(), message.upcast()],
                        );
                        Ok(())
                    });
                }
                Event::WindowEvent {
                    event, window_id, ..
                } => match event {
                    WindowEvent::CloseRequested => {
                        let _ = channel
                            .send(move |mut cx| {
                                let this = cx.undefined();
                                let callback = listener_cb.to_inner(&mut cx);
                                let event = cx.string("close-window");
                                let window_id_boxed = cx.boxed(WindowIdBoxed { window_id });
                                let _ = callback.call(
                                    &mut cx,
                                    this,
                                    &[event.upcast(), window_id_boxed.upcast()],
                                );
                                Ok(())
                            })
                            .join();
                    }
                    WindowEvent::Resized(_) => {
                        let webview = webviews.get(&window_id).unwrap();
                        webview.resize().unwrap();
                    }
                    _ => {}
                },
                _ => (),
            }
        });
    });
    Ok(cx.undefined())
}

fn create_new_window_js(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<IpcBoxed>>(0)?;
    let title = cx.argument::<JsString>(1)?.value(&mut cx);
    let devtools = cx.argument::<JsBoolean>(2)?.value(&mut cx);
    let transparent = cx.argument::<JsBoolean>(3)?.value(&mut cx);
    let frameless = cx.argument::<JsBoolean>(4)?.value(&mut cx);
    let width = cx.argument::<JsNumber>(5)?.value(&mut cx) as u32;
    let height = cx.argument::<JsNumber>(6)?.value(&mut cx) as u32;
    let visible = cx.argument::<JsBoolean>(7)?.value(&mut cx);
    let resizable = cx.argument::<JsBoolean>(8)?.value(&mut cx);
    let initialization_script = cx.argument::<JsString>(9)?.value(&mut cx);
    let cb = cx.argument::<JsFunction>(10)?.root(&mut cx);

    let option = Options {
        title,
        width,
        height,
        visible,
        devtools,
        frameless,
        resizable,
        transparent,
        initialization_script,
    };
    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let _ = proxy.send_event(UserEvents::CreateNewWindow(option, cb));
    Ok(cx.undefined())
}

fn close_window(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<IpcBoxed>>(0)?;
    let window_id = cx.argument::<JsBox<WindowIdBoxed>>(1)?;
    let cb = cx.argument::<JsFunction>(2)?.root(&mut cx);

    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let window_id = window_id.deref();
    let window_id = window_id.window_id.clone();

    let _ = proxy.send_event(UserEvents::CloseWindow(window_id, cb));
    Ok(cx.undefined())
}

fn center_window(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<IpcBoxed>>(0)?;
    let window_id = cx.argument::<JsBox<WindowIdBoxed>>(1)?;
    let cb = cx.argument::<JsFunction>(2)?.root(&mut cx);

    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let window_id = window_id.deref();
    let window_id = window_id.window_id.clone();

    let _ = proxy.send_event(UserEvents::CenterWindow(window_id, cb));
    Ok(cx.undefined())
}

fn set_title_window(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<IpcBoxed>>(0)?;
    let window_id = cx.argument::<JsBox<WindowIdBoxed>>(1)?;
    let title = cx.argument::<JsString>(2)?.value(&mut cx);
    let cb = cx.argument::<JsFunction>(3)?.root(&mut cx);

    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let window_id = window_id.deref();
    let window_id = window_id.window_id.clone();

    let _ = proxy.send_event(UserEvents::ChangeTitleWindow(window_id, title, cb));
    Ok(cx.undefined())
}

fn set_resizable_window(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<IpcBoxed>>(0)?;
    let window_id = cx.argument::<JsBox<WindowIdBoxed>>(1)?;
    let resizable = cx.argument::<JsBoolean>(2)?.value(&mut cx);
    let cb = cx.argument::<JsFunction>(3)?.root(&mut cx);

    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let window_id = window_id.deref();
    let window_id = window_id.window_id.clone();

    let _ = proxy.send_event(UserEvents::SetResizableWindow(window_id, resizable, cb));
    Ok(cx.undefined())
}

fn set_visible_window(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<IpcBoxed>>(0)?;
    let window_id = cx.argument::<JsBox<WindowIdBoxed>>(1)?;
    let visible = cx.argument::<JsBoolean>(2)?.value(&mut cx);
    let cb = cx.argument::<JsFunction>(3)?.root(&mut cx);

    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let window_id = window_id.deref();
    let window_id = window_id.window_id.clone();

    let _ = proxy.send_event(UserEvents::SetVisibleWindow(window_id, visible, cb));
    Ok(cx.undefined())
}

fn evaluate_script(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<IpcBoxed>>(0)?;
    let window_id = cx.argument::<JsBox<WindowIdBoxed>>(1)?;
    let script = cx.argument::<JsString>(2)?.value(&mut cx);
    let cb = cx.argument::<JsFunction>(3)?.root(&mut cx);

    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let window_id = window_id.deref();
    let window_id = window_id.window_id.clone();

    let _ = proxy.send_event(UserEvents::EvaluateScript(window_id, script, cb));
    Ok(cx.undefined())
}

fn set_window_size(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<IpcBoxed>>(0)?;
    let window_id = cx.argument::<JsBox<WindowIdBoxed>>(1)?;
    let width = cx.argument::<JsNumber>(2)?.value(&mut cx) as u32;
    let height = cx.argument::<JsNumber>(3)?.value(&mut cx) as u32;
    let cb = cx.argument::<JsFunction>(4)?.root(&mut cx);

    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let window_id = window_id.deref();
    let window_id = window_id.window_id.clone();

    let _ = proxy.send_event(UserEvents::SetWindowSize(window_id, width, height, cb));
    Ok(cx.undefined())
}

fn get_window_size(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<IpcBoxed>>(0)?;
    let window_id = cx.argument::<JsBox<WindowIdBoxed>>(1)?;
    let cb = cx.argument::<JsFunction>(2)?.root(&mut cx);

    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let window_id = window_id.deref();
    let window_id = window_id.window_id.clone();

    let _ = proxy.send_event(UserEvents::GetWindowSize(window_id, cb));
    Ok(cx.undefined())
}

fn set_minimized_window(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<IpcBoxed>>(0)?;
    let window_id = cx.argument::<JsBox<WindowIdBoxed>>(1)?;
    let minimized = cx.argument::<JsBoolean>(2)?.value(&mut cx);
    let cb = cx.argument::<JsFunction>(3)?.root(&mut cx);

    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let window_id = window_id.deref();
    let window_id = window_id.window_id.clone();

    let _ = proxy.send_event(UserEvents::SetMinimizedWindow(window_id, minimized, cb));
    Ok(cx.undefined())
}

fn focus_window(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<IpcBoxed>>(0)?;
    let window_id = cx.argument::<JsBox<WindowIdBoxed>>(1)?;
    let cb = cx.argument::<JsFunction>(2)?.root(&mut cx);

    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let window_id = window_id.deref();
    let window_id = window_id.window_id.clone();

    let _ = proxy.send_event(UserEvents::FocusWindow(window_id, cb));
    Ok(cx.undefined())
}

fn set_always_on_top_window(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<IpcBoxed>>(0)?;
    let window_id = cx.argument::<JsBox<WindowIdBoxed>>(1)?;
    let flag = cx.argument::<JsBoolean>(2)?.value(&mut cx);
    let cb = cx.argument::<JsFunction>(3)?.root(&mut cx);

    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let window_id = window_id.deref();
    let window_id = window_id.window_id.clone();

    let _ = proxy.send_event(UserEvents::SetAlwaysOnTopWindow(window_id, flag, cb));
    Ok(cx.undefined())
}

fn set_ignore_cursor_events(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<IpcBoxed>>(0)?;
    let window_id = cx.argument::<JsBox<WindowIdBoxed>>(1)?;
    let flag = cx.argument::<JsBoolean>(2)?.value(&mut cx);
    let cb = cx.argument::<JsFunction>(3)?.root(&mut cx);

    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let window_id = window_id.deref();
    let window_id = window_id.window_id.clone();

    let _ = proxy.send_event(UserEvents::SetIgnoreCursorEvents(window_id, flag, cb));
    Ok(cx.undefined())
}

fn open_devtools(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<IpcBoxed>>(0)?;
    let window_id = cx.argument::<JsBox<WindowIdBoxed>>(1)?;
    let cb = cx.argument::<JsFunction>(2)?.root(&mut cx);

    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let window_id = window_id.deref();
    let window_id = window_id.window_id.clone();

    let _ = proxy.send_event(UserEvents::OpenDevtools(window_id, cb));
    Ok(cx.undefined())
}

fn close_devtools(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<IpcBoxed>>(0)?;
    let window_id = cx.argument::<JsBox<WindowIdBoxed>>(1)?;
    let cb = cx.argument::<JsFunction>(2)?.root(&mut cx);

    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let window_id = window_id.deref();
    let window_id = window_id.window_id.clone();

    let _ = proxy.send_event(UserEvents::CloseDevtools(window_id, cb));
    Ok(cx.undefined())
}

fn set_frameless_window(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<IpcBoxed>>(0)?;
    let window_id = cx.argument::<JsBox<WindowIdBoxed>>(1)?;
    let flag = cx.argument::<JsBoolean>(2)?.value(&mut cx);
    let cb = cx.argument::<JsFunction>(3)?.root(&mut cx);

    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let window_id = window_id.deref();
    let window_id = window_id.window_id.clone();

    let _ = proxy.send_event(UserEvents::SetFramelessWindow(window_id, flag, cb));
    Ok(cx.undefined())
}

fn set_window_icon(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<IpcBoxed>>(0)?;
    let window_id = cx.argument::<JsBox<WindowIdBoxed>>(1)?;
    let rgba = cx.argument::<JsBuffer>(2)?.as_slice(&cx).to_vec();
    let width = cx.argument::<JsNumber>(3)?.value(&mut cx) as u32;
    let height = cx.argument::<JsNumber>(4)?.value(&mut cx) as u32;
    let cb = cx.argument::<JsFunction>(5)?.root(&mut cx);

    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let window_id = window_id.deref();
    let window_id = window_id.window_id.clone();

    let _ = proxy.send_event(UserEvents::SetWindowIcon(
        window_id, rgba, width, height, cb,
    ));
    Ok(cx.undefined())
}

fn compare_window_id(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let window_id_a = cx.argument::<JsBox<WindowIdBoxed>>(0)?;
    let window_id_b = cx.argument::<JsBox<WindowIdBoxed>>(1)?;

    let window_id_a = window_id_a.deref();
    let window_id_a = window_id_a.window_id.clone();

    let window_id_b = window_id_b.deref();
    let window_id_b = window_id_b.window_id.clone();
    Ok(cx.boolean(window_id_a == window_id_b))
}

fn unsafe_quit(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<IpcBoxed>>(0)?;
    let cb = cx.argument::<JsFunction>(1)?.root(&mut cx);

    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();

    let _ = proxy.send_event(UserEvents::UnsafeQuit(cb));
    Ok(cx.undefined())
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("app_init", app_init)?;
    cx.export_function("create_new_window", create_new_window_js)?;
    cx.export_function("close_window", close_window)?;
    cx.export_function("center_window", center_window)?;
    cx.export_function("set_title_window", set_title_window)?;
    cx.export_function("set_resizable_window", set_resizable_window)?;
    cx.export_function("set_visible_window", set_visible_window)?;
    cx.export_function("evaluate_script", evaluate_script)?;
    cx.export_function("set_window_size", set_window_size)?;
    cx.export_function("get_window_size", get_window_size)?;
    cx.export_function("set_minimized_window", set_minimized_window)?;
    cx.export_function("focus_window", focus_window)?;
    cx.export_function("set_always_on_top_window", set_always_on_top_window)?;
    cx.export_function("set_ignore_cursor_events", set_ignore_cursor_events)?;
    cx.export_function("open_devtools", open_devtools)?;
    cx.export_function("close_devtools", close_devtools)?;
    cx.export_function("set_frameless_window", set_frameless_window)?;
    cx.export_function("set_window_icon", set_window_icon)?;
    cx.export_function("compare_window_id", compare_window_id)?;
    cx.export_function("unsafe_quit", unsafe_quit)?;
    Ok(())
}
