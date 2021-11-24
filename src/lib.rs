#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use napi::{
  threadsafe_function::{ThreadSafeCallContext, ThreadsafeFunctionCallMode},
  CallContext, Env, JsFunction, JsNumber, JsObject, JsUndefined, Property, Result, Task,
};
use std::convert::TryInto;

#[cfg(all(
  any(windows, unix),
  target_arch = "x86_64",
  not(target_env = "musl"),
  not(debug_assertions)
))]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[module_exports]
fn init(mut exports: JsObject, env: Env) -> Result<()> {
  exports.create_named_method("add", add)?;
  exports.create_named_method("sync", sync_fn)?;
  exports.create_named_method("sleep", sleep)?;
  exports.create_named_method("obj", obj)?;
  exports.create_named_method("modifyObj", modify_obj)?;
  exports.create_named_method("modifyArr", modify_arr)?;
  let test_class = env.define_class(
    "TestClass",
    class_constructor,
    &[
      Property::new(&env, "addCount")?.with_method(add_count),
      Property::new(&env, "addNativeCount")?.with_method(add_native_count),
    ],
  )?;
  exports.set_named_property("TestClass", test_class)?;
  exports.create_named_method("addCb", add_cb)?;
  Ok(())
}

struct TestClass {
  value: i32,
}

#[js_function(1)]
fn class_constructor(ctx: CallContext) -> Result<JsUndefined> {
  let count: i32 = ctx.get::<JsNumber>(0)?.try_into()?;
  let mut this: JsObject = ctx.this_unchecked();
  ctx.env.wrap(&mut this, TestClass { value: count + 100 })?;
  this.set_named_property("count", ctx.env.create_int32(count)?)?;
  ctx.env.get_undefined()
}

#[js_function(1)]
fn add_count(ctx: CallContext) -> Result<JsNumber> {
  let add: i32 = ctx.get::<JsNumber>(0)?.try_into()?;
  let mut this: JsObject = ctx.this_unchecked();
  let count: i32 = this.get_named_property::<JsNumber>("count")?.try_into()?;
  this.set_named_property("count", ctx.env.create_int32(count + add)?)?;
  this.get_named_property("count")
}

#[js_function(1)]
fn add_native_count(ctx: CallContext) -> Result<JsNumber> {
  let add: i32 = ctx.get::<JsNumber>(0)?.try_into()?;
  let this: JsObject = ctx.this_unchecked();
  let test_class: &mut TestClass = ctx.env.unwrap(&this)?;
  test_class.value += add;
  ctx.env.create_int32(test_class.value)
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
fn modify_arr(ctx: CallContext) -> Result<JsObject> {
  let input = ctx.get::<JsObject>(0)?;
  let vec: Vec<u32> = ctx.env.from_js_value(input)?;
  let mut arr = ctx.env.create_array()?;
  for (index, elem) in vec.iter().enumerate() {
    arr.set_element(index as u32, ctx.env.create_int32((*elem + 100) as i32)?)?;
  }
  Ok(arr)
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

#[derive(Debug)]
struct Message {
  value: String,
  id: i32,
}

#[js_function(1)]
fn add_cb(ctx: CallContext) -> Result<JsUndefined> {
  let cb = ctx.get::<JsFunction>(0)?;
  let tscb =
    ctx
      .env
      .create_threadsafe_function(&cb, 0, |tscx: ThreadSafeCallContext<Message>| {
        let mut obj = tscx.env.create_object()?;
        obj.set_named_property("value", tscx.env.create_string(tscx.value.value.as_str())?)?;
        obj.set_named_property("id", tscx.env.create_int32(tscx.value.id)?)?;
        Ok(vec![obj])
      })?;
  let mut count = 0;
  loop {
    count += 1;
    tscb.call(
      Ok(Message {
        value: "hello message".to_owned(),
        id: 13,
      }),
      ThreadsafeFunctionCallMode::NonBlocking,
    );
    if count > 10 {
      break;
    }
  }
  ctx.env.get_undefined()
}
