const { add, sync, sleep, obj, modifyObj, modifyArr, TestClass } = require('./index')

console.assert(add(1, 2) === 3, 'Add test failed')
console.assert(sync(0) === 100, 'Simple test failed')

// sleep(1).then((res) => {
//   console.info(res)
// })

console.info(obj())

const obj2 = { name: 'xm' }
console.info(modifyObj(obj2))
console.info(modifyArr([1, 2, 3]))

const objClass = new TestClass(1)
console.info(objClass)
console.info(objClass.addCount(100))
console.info(objClass.addNativeCount(100))
console.info(objClass)
