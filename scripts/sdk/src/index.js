module.exports.deploymentRegistry = require("./deploymentRegistry")

// Old-school reexport.
Object.assign(module.exports, require("./constants.js"));
Object.assign(module.exports, require("./abiRegistry.js"));
Object.assign(module.exports, require("./contractRegistry.js"));
Object.assign(module.exports, require("./polkadotWrappers.js"));
module.exports.ddcBucket = require("./bucket/ddcBucket.js");
module.exports.ddcBucketQuery = require("./bucket/ddcBucketQuery.js");
module.exports.ddcNftRegistry = require("./ddcNftRegistry/nftRegistry.js");
module.exports.config = require("./config");
module.exports.lodash = require("lodash");