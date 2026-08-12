//! Android binding para o PS5 Emulator

use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jstring, jint};

#[no_mangle]
pub extern "system" fn Java_com_ps5_emulator_MainActivity_startEmulator(
    env: JNIEnv,
    _class: JClass,
    game_path: JString,
) -> jstring {
    let path: String = env
        .get_string(&game_path)
        .unwrap()
        .into();

    // Inicia o emulador
    let result = format!("Emulador iniciado com: {}", path);
    
    env.new_string(&result).unwrap().into_inner()
}

#[no_mangle]
pub extern "system" fn Java_com_ps5_emulator_MainActivity_stopEmulator(
    _env: JNIEnv,
    _class: JClass,
) {
    // Para o emulador
}

#[no_mangle]
pub extern "system" fn Java_com_ps5_emulator_MainActivity_getVersion(
    _env: JNIEnv,
    _class: JClass,
) -> jstring {
    let version = env.new_string("PS5 Emulator v1.0.0").unwrap();
    version.into_inner()
}
