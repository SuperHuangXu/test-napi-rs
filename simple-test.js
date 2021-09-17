const { add, sync, sleep, obj, modifyObj, modifyArr } = require('./index')

console.assert(add(1, 2) === 3, 'Add test failed')
console.assert(sync(0) === 100, 'Simple test failed')

// sleep(1).then((res) => {
//   console.info(res)
// })

console.info(obj())

const obj2 = { name: 'xm' }
console.info(modifyObj(obj2))
console.info(modifyArr([1, 2, 3]))
