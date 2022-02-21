require("./deployments").init();

// Old-school reexport.
Object.assign(module.exports, require("./constants.js"));
Object.assign(module.exports, require("./abiRegistry.js"));
Object.assign(module.exports, require("./contractRegistry.js"));
Object.assign(module.exports, require("./polkadotWrappers.js"));
module.exports.ddcBucket = require("./ddcBucket.js");
