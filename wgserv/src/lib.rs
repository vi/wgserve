use jni::objects::{JClass, JString};
use jni::sys::jlong;
use jni::sys::jstring;
use jni::JNIEnv;
use std::ptr::null_mut;
use tracing::{debug, info};
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;

/*
JNIEXPORT jstring JNICALL Java_org_vi_1server_wgserver_Native_setConfig
JNIEXPORT jstring JNICALL Java_org_vi_1server_wgserver_Native_run
*/

struct App {
    config: Option<Config>,
}

impl App {
    pub fn new() -> App {
        App { config: None }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct Config {
    #[serde(default)]
    debug: bool,
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
    let app = unsafe { Box::from_raw(instance as usize as *mut App) };
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
            let mut app = unsafe { Box::from_raw(instance as usize as *mut App) };
            app.config = Some(x);
            let _ = Box::into_raw(app);
            null_mut()
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

    info!("Hello");
    debug!("Debugging is enabled");

    std::thread::sleep_ms(10000);

    app.config = Some(config);
    let _ = Box::into_raw(app);
    null_mut()
}

#[no_mangle]
pub extern "system" fn Java_org_vi_1server_wgserver_Native_getSampleConfig(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let sample_config = Config { debug: false };
    let output = env
        .new_string(toml::to_string_pretty(&sample_config).unwrap())
        .expect("Couldn't create java string!");
    output.into_raw()
}
