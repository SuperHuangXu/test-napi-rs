#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use napi::{CallContext, Env, JsNumber, JsObject, JsUndefined, Result, Task};
use std::convert::TryInto;
use serde_json::{from_str, to_string};

#[cfg(all(
  any(windows, unix),
  target_arch = "x86_64",
  not(target_env = "musl"),
  not(debug_assertions)
))]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[module_exports]
fn init(mut exports: JsObject) -> Result<()> {
  exports.create_named_method("add", add)?;
  exports.create_named_method("sync", sync_fn)?;
  exports.create_named_method("sleep", sleep)?;
  exports.create_named_method("obj", obj)?;
  exports.create_named_method("modifyObj", modify_obj)?;
  exports.create_named_method("modifyArr", modify_arr)?;
  Ok(())
}

// rust 返回 js 对象
#[js_function]
fn obj(ctx: CallContext) -> Result<JsObject> {
  let mut obj_value = ctx.env.create_object()?;
  obj_value.set_named_property("name", ctx.env.create_string("xm")?)?;
  obj_value.set_named_property("age", ctx.env.create_int32(12)?)?;
  obj_value.set_property(ctx.env.create_string("hello")?, ctx.env.create_int32(12)?)?;
  Ok(obj_value)
}

// rust 操作 js 传的对象
#[js_function(1)]
fn modify_obj(ctx: CallContext) -> Result<JsObject> {
  let mut obj_value = ctx.get::<JsObject>(0)?;
  obj_value.set_named_property("name", ctx.env.create_string("rust modify...")?)?;
  Ok(obj_value)
}

// rust 修改数组
#[js_function(1)]
fn modify_arr(ctx: CallContext) -> Result<JsUndefined> {
  let input = ctx.get::<JsObject>(0)?;
  let _: Vec<u32> = ctx.env.from_js_value(input)?;
  ctx.env.get_undefined()
}

// 传两个参数
#[js_function(2)]
fn add(ctx: CallContext) -> Result<JsNumber> {
  let a = ctx.get::<JsNumber>(0)?.get_uint32()?;
  let b = ctx.get::<JsNumber>(1)?.get_uint32()?;
  ctx.env.create_uint32(a + b)
}

#[js_function(1)]
fn sync_fn(ctx: CallContext) -> Result<JsNumber> {
  let argument: u32 = ctx.get::<JsNumber>(0)?.try_into()?;
  ctx.env.create_uint32(argument + 100)
}

struct AsyncTask(u32);

impl Task for AsyncTask {
  type Output = u32;
  type JsValue = JsNumber;

  fn compute(&mut self) -> Result<Self::Output> {
    use std::thread::sleep;
    use std::time::Duration;
    sleep(Duration::from_millis(self.0 as u64));
    Ok(self.0 * 2)
  }

  fn resolve(self, env: Env, output: Self::Output) -> Result<Self::JsValue> {
    env.create_uint32(output)
  }
}

// 返回 Promise
#[js_function(1)]
fn sleep(ctx: CallContext) -> Result<JsObject> {
  let argument: u32 = ctx.get::<JsNumber>(0)?.try_into()?;
  let task = AsyncTask(argument);
  let async_task = ctx.env.spawn(task)?;
  Ok(async_task.promise_object())
}
