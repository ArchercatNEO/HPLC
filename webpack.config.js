const path = require('path');
const glob = require("glob");

module.exports = {
    entry: {
        js: glob.sync("./scripts/*.js"),  
     }, // Path to the folder
    output: {
        filename: 'bundle.js', // Output bundled file name
        path: path.resolve(__dirname, 'dist'), // Output directory
    },
};