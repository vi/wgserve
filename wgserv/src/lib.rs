use jni::objects::{JClass, JString};
use jni::sys::jlong;
use jni::sys::jstring;
use jni::JNIEnv;
use std::net::{IpAddr, SocketAddr};
use std::ptr::null_mut;
use tracing::{info, error};
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;

/*
JNIEXPORT jstring JNICALL Java_org_vi_1server_wgserver_Native_setConfig
JNIEXPORT jstring JNICALL Java_org_vi_1server_wgserver_Native_run
*/

struct App {
    config: Option<Config>,
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
}

impl App {
    pub fn new() -> App {
        App {
            config: None,
            shutdown: None,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct Config {
    #[serde(default)]
    debug: bool,

    pub private_key: String,
    pub peer_key: String,
    pub peer_endpoint: Option<SocketAddr>,
    pub keepalive_interval: Option<u16>,
    pub bind_ip_port: SocketAddr,

    pub dns_addr: Option<SocketAddr>,
    pub pingable: Option<IpAddr>,
    pub mtu: usize,
    pub tcp_buffer_size: usize,
    pub incoming_udp: Vec<PortForward>,
    pub incoming_tcp: Vec<PortForward>,

    pub transmit_queue_capacity: usize,
}
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct PortForward {
    pub host: SocketAddr,
    pub src: Option<SocketAddr>,
    pub dst: SocketAddr,
}
impl From<PortForward> for libwgslirpy::router::PortForward {
    fn from(value: PortForward) -> Self {
        libwgslirpy::router::PortForward {
            host: value.host,
            src: value.src,
            dst: value.dst
        }
    }
}
#[no_mangle]
pub extern "system" fn Java_org_vi_1server_wgserver_Native_create(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    Box::into_raw(Box::new(App::new())) as usize as jlong
}

#[no_mangle]
pub extern "system" fn Java_org_vi_1server_wgserver_Native_destroy(
    _env: JNIEnv,
    _class: JClass,
    instance: jlong,
) {
    let mut app = unsafe { Box::from_raw(instance as usize as *mut App) };
    if let Some(shutdown) = app.shutdown.take() {
        let _ = shutdown.send(());
    }
    drop(app);
}

#[no_mangle]
pub extern "system" fn Java_org_vi_1server_wgserver_Native_setConfig(
    mut env: JNIEnv,
    _class: JClass,
    instance: jlong,
    input: JString,
) -> jstring {
    let input: String = env
        .get_string(&input)
        .expect("Couldn't get java string!")
        .into();

    match toml::from_str::<Config>(&input) {
        Ok(x) => {
            let mut failure: Option<&'static str> = None;

            if libwgslirpy::parsebase64_32(&x.peer_key).is_err() {
                failure = Some("Invalid peer_key")
            }
            if libwgslirpy::parsebase64_32(&x.private_key).is_err() {
                failure = Some("Invalid private_key")
            }

            if let Some(f) = failure {
                let output = env
                    .new_string(format!("{}", f))
                    .expect("Couldn't create java string!");
                output.into_raw()
            } else {
                let mut app = unsafe { Box::from_raw(instance as usize as *mut App) };
                app.config = Some(x);
                let _ = Box::into_raw(app);
                null_mut()
            }
        }
        Err(e) => {
            let output = env
                .new_string(format!("{}", e))
                .expect("Couldn't create java string!");
            output.into_raw()
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_org_vi_1server_wgserver_Native_run(
    env: JNIEnv,
    _class: JClass,
    instance: jlong,
) -> jstring {
    let mut app = unsafe { Box::from_raw(instance as usize as *mut App) };

    let Some(config) = app.config else {
        let _ = Box::into_raw(app);
        return env.new_string("setConfig should precede run").unwrap().into_raw()
    };

    let (tx, rx_shutdown) = tokio::sync::oneshot::channel();
    app.shutdown = Some(tx);
    app.config = None;
    let _ = Box::into_raw(app);

    let _tracing = {
        let s = tracing_subscriber::registry();
        let a = tracing_android::layer("WgServer").unwrap();
        let lf: Option<_> = if !config.debug {
            Some(tracing_subscriber::filter::LevelFilter::INFO)
        } else {
            None
        };
        tracing::subscriber::set_default(s.with(a).with(lf))
    };

    let router_config = libwgslirpy::router::Opts {
        dns_addr: config.dns_addr,
        pingable: config.pingable,
        mtu: config.mtu,
        tcp_buffer_size: config.tcp_buffer_size,
        incoming_udp: config.incoming_udp.into_iter().map(|x|x.into()).collect(),
        incoming_tcp: config.incoming_tcp.into_iter().map(|x|x.into()).collect(),
    };
    let wg_config = libwgslirpy::wg::Opts {
        private_key: libwgslirpy::parsebase64_32(&config.private_key).unwrap().into(),
        peer_key: libwgslirpy::parsebase64_32(&config.peer_key).unwrap().into(),
        peer_endpoint: config.peer_endpoint,
        keepalive_interval: config.keepalive_interval,
        bind_ip_port: config.bind_ip_port,
    };

    let rt = tokio::runtime::Builder::new_current_thread().enable_io().enable_time().build().unwrap();

    let ret = rt.block_on(async move {
        let f = libwgslirpy::run(wg_config, router_config, config.transmit_queue_capacity);
        let mut jh = tokio::spawn(f);
        let mut rx_shutdown = rx_shutdown;
        info!("Starting wgslirpy");
        loop {
            enum SelectOutcome {
                Returned(Result<anyhow::Result<()>,tokio::task::JoinError>),
                Aborted,
            }
            let ret = tokio::select! {
                x = &mut jh => SelectOutcome::Returned(x),
                _ = &mut rx_shutdown => SelectOutcome::Aborted,
            };
            match ret {
                SelectOutcome::Returned(Ok(Err(e))) => {
                    error!("Failed to run wgslirpy: {e}");
                    return format!("{e}");
                }
                SelectOutcome::Returned(_) => {
                    error!("Abnormal exit");
                    return format!("Abnormal exit");
                }
                SelectOutcome::Aborted => {
                    jh.abort();
                    return "".to_owned();
                }
            }
        }
    });

    return env.new_string(ret).unwrap().into_raw()
}

#[no_mangle]
pub extern "system" fn Java_org_vi_1server_wgserver_Native_getSampleConfig(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let sample_config = Config {
        debug: false,
        private_key: "SG43Zi0wGp4emfJ/XpTnnmtnK8SSjjIHOc3Zh37c928=".to_owned(),
        peer_key: "rPpCjWzIv/yAtZZi+C/pVprie8D0QaGlPtJXlDi6bmI=".to_owned(),
        peer_endpoint: Some("192.168.0.185:9796".parse().unwrap()),
        keepalive_interval: Some(15),
        bind_ip_port: "0.0.0.0:9797".parse().unwrap(),
        dns_addr: Some("10.0.2.1:53".parse().unwrap()),
        pingable: Some("10.0.2.1".parse().unwrap()),
        mtu: 1420,
        tcp_buffer_size: 65536,
        incoming_udp: vec![PortForward {
            host: "0.0.0.0:8053".parse().unwrap(),
            src: Some("99.99.99.99:99".parse().unwrap()),
            dst: "10.0.2.15:5353".parse().unwrap(),
        }],
        incoming_tcp: vec![
            PortForward {
                host: "0.0.0.0:8080".parse().unwrap(),
                src: None,
                dst: "10.0.2.15:80".parse().unwrap(),
            },
            PortForward {
                host: "0.0.0.0:2222".parse().unwrap(),
                src: None,
                dst: "10.0.2.15:22".parse().unwrap(),
            },
        ],
        transmit_queue_capacity: 128,
    };
    let output = env
        .new_string(toml::to_string_pretty(&sample_config).unwrap())
        .expect("Couldn't create java string!");
    output.into_raw()
}
