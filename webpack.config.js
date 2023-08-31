const path = require('path');

module.exports = {
    entry: {
        js: ["./scripts/chart.js", "./scripts/analysis.js"],  
     }, // Path to the folder
    output: {
        filename: 'bundle.js', // Output bundled file name
        path: path.resolve(__dirname, 'dist'), // Output directory
        library: 'Server',
    },
};