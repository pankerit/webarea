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
    CloseWindow,
    Center,
    ChangeTitle(String),
    VisibleWindow(bool),
    ResizableWindow(bool),
    EvaluateScript(String),
    SetInnerSize(u32, u32),
    GetInnerSize,
    DragWindow,
    SetFocus,
    // IgnoreCursorEvents(bool),
    SetWindowIcon(Vec<u8>, u32, u32),
    OpenDevtools,
    IpcPostMessage(String),
}

struct Ipc {
    proxy: Arc<EventLoopProxy<UserEvents>>,
}

impl Finalize for Ipc {}

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
                Event::UserEvent(UserEvents::CloseWindow) => {
                    *control_flow = ControlFlow::Exit;
                }
                Event::UserEvent(UserEvents::Center) => {
                    let window = webview.window();
                    if let Some(monitor) = window.current_monitor() {
                        let screen_size = monitor.size();
                        let window_size = window.inner_size();
                        let x = (screen_size.width - window_size.width) / 2;
                        let y = (screen_size.height - window_size.height) / 2;
                        window.set_outer_position(PhysicalPosition::new(x, y));
                    } else {
                        panic!("Could not get current monitor");
                    }
                }
                Event::UserEvent(UserEvents::ChangeTitle(title)) => {
                    webview.window().set_title(&title);
                }
                Event::UserEvent(UserEvents::SetInnerSize(width, height)) => {
                    webview
                        .window()
                        .set_inner_size(Size::new(PhysicalSize::new(width, height)));
                }
                Event::UserEvent(UserEvents::GetInnerSize) => {
                    let size = webview.window().inner_size();
                    channel.send(move |mut cx| {
                        let this = cx.undefined();
                        let callback = cb.to_inner(&mut cx);
                        let event_type = cx.string("getInnerSize");
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
                    });
                }
                Event::UserEvent(UserEvents::SetWindowIcon(rgba, width, height)) => {
                    let icon = Icon::from_rgba(rgba, width, height);
                    match icon {
                        Ok(icon) => {
                            webview.window().set_window_icon(Some(icon));
                        }
                        Err(err) => {
                            panic!("{}", err);
                        }
                    }
                }
                Event::UserEvent(UserEvents::OpenDevtools) => {
                    webview.open_devtools();
                }
                Event::UserEvent(UserEvents::DragWindow) => {
                    webview.window().drag_window();
                }
                Event::UserEvent(UserEvents::SetFocus) => {
                    webview.window().set_focus();
                }
                Event::UserEvent(UserEvents::VisibleWindow(visible)) => {
                    webview.window().set_visible(visible);
                }
                Event::UserEvent(UserEvents::ResizableWindow(resizable)) => {
                    webview.window().set_resizable(resizable);
                }
                Event::UserEvent(UserEvents::EvaluateScript(script)) => {
                    let _ = webview.evaluate_script(&script);
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
    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let _ = proxy.send_event(UserEvents::OpenDevtools);
    Ok(cx.undefined())
}

fn close(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<Ipc>>(0)?;
    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let _ = proxy.send_event(UserEvents::CloseWindow);
    Ok(cx.undefined())
}

fn set_window_icon(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<Ipc>>(0)?;
    let rgba = cx.argument::<JsBuffer>(1)?.as_slice(&cx).to_vec();
    let width = cx.argument::<JsNumber>(2)?.value(&mut cx) as u32;
    let height = cx.argument::<JsNumber>(3)?.value(&mut cx) as u32;
    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let _ = proxy.send_event(UserEvents::SetWindowIcon(rgba, width, height));
    Ok(cx.undefined())
}

fn drag_window(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<Ipc>>(0)?;
    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let _ = proxy.send_event(UserEvents::DragWindow);
    Ok(cx.undefined())
}

fn set_visible(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<Ipc>>(0)?;
    let flag = cx.argument::<JsBoolean>(1)?.value(&mut cx);
    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let _ = proxy.send_event(UserEvents::VisibleWindow(flag));
    Ok(cx.undefined())
}

fn evaluate_script(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<Ipc>>(0)?;
    let script = cx.argument::<JsString>(1)?.value(&mut cx);
    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let _ = proxy.send_event(UserEvents::EvaluateScript(script));
    Ok(cx.undefined())
}

fn set_focus(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<Ipc>>(0)?;
    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let _ = proxy.send_event(UserEvents::SetFocus);
    Ok(cx.undefined())
}

fn set_title(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<Ipc>>(0)?;
    let title = cx.argument::<JsString>(1)?.value(&mut cx);
    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let _ = proxy.send_event(UserEvents::ChangeTitle(title));
    Ok(cx.undefined())
}

fn set_inner_size(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<Ipc>>(0)?;
    let width = cx.argument::<JsNumber>(1)?.value(&mut cx) as u32;
    let height = cx.argument::<JsNumber>(2)?.value(&mut cx) as u32;
    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let _ = proxy.send_event(UserEvents::SetInnerSize(width, height));
    Ok(cx.undefined())
}

fn get_inner_size(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<Ipc>>(0)?;
    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let _ = proxy.send_event(UserEvents::GetInnerSize);
    Ok(cx.undefined())
}

fn set_resizable(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<Ipc>>(0)?;
    let flag = cx.argument::<JsBoolean>(1)?.value(&mut cx);
    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let _ = proxy.send_event(UserEvents::ResizableWindow(flag));
    Ok(cx.undefined())
}

fn set_center(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let proxy = cx.argument::<JsBox<Ipc>>(0)?;
    let proxy = proxy.deref();
    let proxy = proxy.proxy.clone();
    let _ = proxy.send_event(UserEvents::Center);
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
    cx.export_function("drag_window", drag_window)?;
    cx.export_function("set_visible", set_visible)?;
    cx.export_function("set_focus", set_focus)?;
    cx.export_function("set_title", set_title)?;
    cx.export_function("open_devtools", open_devtools)?;
    cx.export_function("set_inner_size", set_inner_size)?;
    cx.export_function("get_inner_size", get_inner_size)?;
    cx.export_function("set_resizable", set_resizable)?;
    cx.export_function("set_center", set_center)?;
    cx.export_function("evaluate_script", evaluate_script)?;
    Ok(())
}
