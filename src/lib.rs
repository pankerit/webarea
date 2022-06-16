use neon::{prelude::*, types::buffer::TypedArray};
use std::{collections::HashMap, ops::Deref, sync::Arc};
use wry::{
    application::{
        dpi::{PhysicalPosition, PhysicalSize, Size},
        event::{Event, StartCause, WindowEvent},
        event_loop::{ControlFlow, EventLoop, EventLoopProxy},
        platform::windows::EventLoopExtWindows,
        window::{self, Icon, Window, WindowBuilder},
    },
    webview::{WebContext, WebView, WebViewBuilder},
};

enum UserEvents {
    CloseWindow(Root<JsFunction>),
    Center(Root<JsFunction>),
    ChangeTitle(String, Root<JsFunction>),
    VisibleWindow(bool, Root<JsFunction>),
    ResizableWindow(bool, Root<JsFunction>),
    EvaluateScript(String, Root<JsFunction>),
    SetInnerSize(u32, u32, Root<JsFunction>),
    GetInnerSize(Root<JsFunction>),
    DragWindow,
    SetFocus(Root<JsFunction>),
    SetAlwaysOnTop(bool, Root<JsFunction>),
    // IgnoreCursorEvents(bool),
    SetWindowIcon(Vec<u8>, u32, u32, Root<JsFunction>),
    OpenDevtools(Root<JsFunction>),
    IpcPostMessage(String),
}

struct Ipc {
    proxy: Arc<EventLoopProxy<UserEvents>>,
}

impl Finalize for Ipc {}

fn resolve_node_promise(channel: Channel, cb: Root<JsFunction>) {
    channel.send(move |mut cx| {
        let this = cx.undefined();
        let callback = cb.into_inner(&mut cx);
        let _ = callback.call(&mut cx, this, &[]);
        Ok(())
    });
}

fn create(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let cb = cx.argument::<JsFunction>(0)?.root(&mut cx);
    let title = cx.argument::<JsString>(1)?.value(&mut cx);
    let devtools = cx.argument::<JsBoolean>(2)?.value(&mut cx);
    let transparent = cx.argument::<JsBoolean>(3)?.value(&mut cx);
    let frameless = cx.argument::<JsBoolean>(4)?.value(&mut cx);
    let width = cx.argument::<JsNumber>(5)?.value(&mut cx) as u32;
    let height = cx.argument::<JsNumber>(6)?.value(&mut cx) as u32;
    let visible = cx.argument::<JsBoolean>(7)?.value(&mut cx);
    let resizable = cx.argument::<JsBoolean>(8)?.value(&mut cx);
    let initialization_script = cx.argument::<JsString>(9)?.value(&mut cx);

    let (deferred, promise) = cx.promise();
    let cb_arc = Arc::new(cb);
    let channel = cx.channel();
    std::thread::spawn(move || {
        let data_directory = std::env::temp_dir();
        let mut web_context = WebContext::new(Some(data_directory));
        let event_loop: EventLoop<UserEvents> = EventLoop::new_any_thread();
        let proxy = event_loop.create_proxy();

        let p = proxy.clone();
        let handler = move |_window: &Window, req: String| match req.as_str() {
            "drag-window" => {
                let _ = p.send_event(UserEvents::DragWindow);
            }
            _ if req.starts_with("ipc:") => {
                let message = req.replace("ipc:", "");
                let _ = p.send_event(UserEvents::IpcPostMessage(message));
            }
            // "close" => {
            //     let _ = proxy.send_event(UserEvents::CloseWindow());
            // }
            // _ if req.starts_with("change-title") => {
            //     let title = req.replace("change-title:", "");
            //     window.set_title(title.as_str());
            // }
            _ => {
                // let _ = p.send_event(UserEvents::IpcPostMessage(req));
            }
        };

        let window = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(Size::new(PhysicalSize::new(width, height)))
            .with_visible(visible)
            .with_resizable(resizable)
            .with_transparent(transparent)
            .with_decorations(!frameless)
            .build(&event_loop)
            .unwrap();
        let webview = WebViewBuilder::new(window)
            .unwrap()
            .with_initialization_script(&initialization_script)
            .with_transparent(transparent)
            .with_devtools(devtools)
            .with_web_context(&mut web_context)
            .with_ipc_handler(handler)
            .with_html("")
            .unwrap()
            .build()
            .unwrap();

        channel.send(move |mut cx| {
            let boxed = cx.boxed(Ipc {
                proxy: Arc::new(proxy),
            });
            deferred.resolve(&mut cx, boxed);
            Ok(())
        });

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            let cb = cb_arc.clone();
            match event {
                Event::UserEvent(UserEvents::CloseWindow(cb)) => {
                    *control_flow = ControlFlow::Exit;
                    resolve_node_promise(channel.clone(), cb);
                }
                Event::UserEvent(UserEvents::IpcPostMessage(payload)) => {
                    channel.send(move |mut cx| {
                        let this = cx.undefined();
                        let callback = cb.to_inner(&mut cx);
                        let event_type = cx.string("ipc");
                        let event_data = cx.string(payload);
                        let _ = callback.call(
                            &mut cx,
                            this,
                            &[event_type.upcast(), event_data.upcast()],
                        );
                        Ok(())
                    });
                }

                Event::UserEvent(UserEvents::Center(cb)) => {
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
                Event::UserEvent(UserEvents::ChangeTitle(title, cb)) => {
                    webview.window().set_title(&title);
                    resolve_node_promise(channel.clone(), cb);
                }
                Event::UserEvent(UserEvents::SetAlwaysOnTop(flag, cb)) => {
                    webview.window().set_always_on_top(flag);
                    resolve_node_promise(channel.clone(), cb);
                }
                Event::UserEvent(UserEvents::SetInnerSize(width, height, cb)) => {
                    webview
                        .window()
                        .set_inner_size(Size::new(PhysicalSize::new(width, height)));
                    resolve_node_promise(channel.clone(), cb);
                }
                Event::UserEvent(UserEvents::GetInnerSize(cb)) => {
                    let size = webview.window().inner_size();
                    channel.send(move |mut cx| {
                        let this = cx.undefined();
                        let callback = cb.into_inner(&mut cx);
                        let event_data = cx.empty_object();
                        let width = cx.number(size.width as f64);
                        let height = cx.number(size.height as f64);
                        event_data.set(&mut cx, "width", width).unwrap();
                        event_data.set(&mut cx, "height", height).unwrap();
                        let _ = callback.call(&mut cx, this, &[event_data.upcast()]);
                        Ok(())
                    });
                }
                Event::UserEvent(UserEvents::SetWindowIcon(rgba, width, height, cb)) => {
                    let icon = Icon::from_rgba(rgba, width, height);
                    match icon {
                        Ok(icon) => {
                            webview.window().set_window_icon(Some(icon));
                            resolve_node_promise(channel.clone(), cb);
                        }
                        Err(err) => {
                            panic!("{}", err);
                        }
                    }
                }
                Event::UserEvent(UserEvents::OpenDevtools(cb)) => {
                    webview.open_devtools();
                    resolve_node_promise(channel.clone(), cb);
                }
                Event::UserEvent(UserEvents::DragWindow) => {
                    let _ = webview.window().drag_window();
                }
                Event::UserEvent(UserEvents::SetFocus(cb)) => {
                    webview.window().set_focus();
                    resolve_node_promise(channel.clone(), cb);
                }
                Event::UserEvent(UserEvents::VisibleWindow(visible, cb)) => {
                    webview.window().set_visible(visible);
                    resolve_node_promise(channel.clone(), cb);
                }
                Event::UserEvent(UserEvents::ResizableWindow(resizable, cb)) => {
                    webview.window().set_resizable(resizable);
                    resolve_node_promise(channel.clone(), cb);
                }
                Event::UserEvent(UserEvents::EvaluateScript(script, cb)) => {
                    let _ = webview.evaluate_script(&script);
                    resolve_node_promise(channel.clone(), cb);
                }
                Event::NewEvents(StartCause::Init) => println!("Wry has started!"),
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Resized(size) => {
                        if let Err(e) = webview.resize() {
                            panic!("{}", e);
                        }
                        let _ = webview.resize();
                        let _ = channel
                            .send(move |mut cx| {
                                let this = cx.undefined();
                                let callback = cb.to_inner(&mut cx);
                                let event_type = cx.string("resize");
                                let event_data = cx.empty_object();
                                let width = cx.number(size.width as f64);
                                let height = cx.number(size.height as f64);
                                event_data.set(&mut cx, "width", width).unwrap();
                                event_data.set(&mut cx, "height", height).unwrap();
                                let _ = callback.call(
                                    &mut cx,
                                    this,
                                    &[event_type.upcast(), event_data.upcast()],
                                );
                                Ok(())
                            })
                            .join()
                            .unwrap();
                    }
                    _ => {}
                },
                _ => (),
            }
        });
    });
    Ok(promise)
}

fn open_devtools(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<Ipc>>(0)?;
    let cb = cx.argument::<JsFunction>(1)?.root(&mut cx);
    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let _ = proxy.send_event(UserEvents::OpenDevtools(cb));
    Ok(cx.undefined())
}

fn close(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<Ipc>>(0)?;
    let cb = cx.argument::<JsFunction>(1)?.root(&mut cx);
    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let _ = proxy.send_event(UserEvents::CloseWindow(cb));
    Ok(cx.undefined())
}

fn set_window_icon(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<Ipc>>(0)?;
    let rgba = cx.argument::<JsBuffer>(1)?.as_slice(&cx).to_vec();
    let width = cx.argument::<JsNumber>(2)?.value(&mut cx) as u32;
    let height = cx.argument::<JsNumber>(3)?.value(&mut cx) as u32;
    let cb = cx.argument::<JsFunction>(4)?.root(&mut cx);
    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let _ = proxy.send_event(UserEvents::SetWindowIcon(rgba, width, height, cb));
    Ok(cx.undefined())
}

fn set_visible(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<Ipc>>(0)?;
    let flag = cx.argument::<JsBoolean>(1)?.value(&mut cx);
    let cb = cx.argument::<JsFunction>(2)?.root(&mut cx);
    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let _ = proxy.send_event(UserEvents::VisibleWindow(flag, cb));
    Ok(cx.undefined())
}

fn evaluate_script(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<Ipc>>(0)?;
    let script = cx.argument::<JsString>(1)?.value(&mut cx);
    let cb = cx.argument::<JsFunction>(2)?.root(&mut cx);
    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let _ = proxy.send_event(UserEvents::EvaluateScript(script, cb));
    Ok(cx.undefined())
}

fn set_focus(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<Ipc>>(0)?;
    let cb = cx.argument::<JsFunction>(1)?.root(&mut cx);
    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let _ = proxy.send_event(UserEvents::SetFocus(cb));
    Ok(cx.undefined())
}

fn set_title(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<Ipc>>(0)?;
    let title = cx.argument::<JsString>(1)?.value(&mut cx);
    let cb = cx.argument::<JsFunction>(2)?.root(&mut cx);
    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let _ = proxy.send_event(UserEvents::ChangeTitle(title, cb));
    Ok(cx.undefined())
}

fn set_inner_size(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<Ipc>>(0)?;
    let width = cx.argument::<JsNumber>(1)?.value(&mut cx) as u32;
    let height = cx.argument::<JsNumber>(2)?.value(&mut cx) as u32;
    let cb = cx.argument::<JsFunction>(3)?.root(&mut cx);
    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let _ = proxy.send_event(UserEvents::SetInnerSize(width, height, cb));
    Ok(cx.undefined())
}

fn get_inner_size(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<Ipc>>(0)?;
    let cb = cx.argument::<JsFunction>(1)?.root(&mut cx);
    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let _ = proxy.send_event(UserEvents::GetInnerSize(cb));
    Ok(cx.undefined())
}

fn set_resizable(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<Ipc>>(0)?;
    let flag = cx.argument::<JsBoolean>(1)?.value(&mut cx);
    let cb = cx.argument::<JsFunction>(2)?.root(&mut cx);
    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let _ = proxy.send_event(UserEvents::ResizableWindow(flag, cb));
    Ok(cx.undefined())
}

fn set_center(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<Ipc>>(0)?;
    let cb = cx.argument::<JsFunction>(1)?.root(&mut cx);
    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let _ = proxy.send_event(UserEvents::Center(cb));
    Ok(cx.undefined())
}

fn set_always_on_top(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<Ipc>>(0)?;
    let flag = cx.argument::<JsBoolean>(1)?.value(&mut cx);
    let cb = cx.argument::<JsFunction>(2)?.root(&mut cx);
    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let _ = proxy.send_event(UserEvents::SetAlwaysOnTop(flag, cb));
    Ok(cx.undefined())
}

// fn set_ignore_cursor_events(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     let proxy = cx.argument::<JsBox<Ipc>>(0)?;
//     let flag = cx.argument::<JsBoolean>(0)?.value(&mut cx);
//     let proxy = proxy.deref();
//     let proxy = proxy.proxy.clone();
//     let _ = proxy.send_event(UserEvents::IgnoreCursorEvents(flag));
//     Ok(cx.undefined())
// }

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("create", create)?;
    cx.export_function("close", close)?;
    cx.export_function("set_window_icon", set_window_icon)?;
    cx.export_function("set_visible", set_visible)?;
    cx.export_function("set_focus", set_focus)?;
    cx.export_function("set_title", set_title)?;
    cx.export_function("open_devtools", open_devtools)?;
    cx.export_function("set_inner_size", set_inner_size)?;
    cx.export_function("get_inner_size", get_inner_size)?;
    cx.export_function("set_resizable", set_resizable)?;
    cx.export_function("set_center", set_center)?;
    cx.export_function("evaluate_script", evaluate_script)?;
    cx.export_function("set_always_on_top", set_always_on_top)?;
    Ok(())
}
