const express = require('express');
const path = require('path');

const app = express();
const port = 3000;

// Serve static files from the 'js' folder

app.use('/scripts', express.static(path.join(__dirname, 'scripts')));
//app.use('/plots', express.static(path.join(__dirname, 'plots')));
  
 

app.use('/dist', express.static(path.join(__dirname, 'dist')));
app.use('/styles', express.static(path.join(__dirname, 'styles')));

// Serve your index.html
app.get('/', (req: any, res: any) => {
    res.sendFile(path.join(__dirname, 'index.html'));
});

app.listen(port, () => {
    console.log(`Server is running on http://localhost:${port}`);
});
