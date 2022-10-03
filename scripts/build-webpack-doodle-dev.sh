set -e
NAME=$1
webpack --mode development --config main.webpack.config.js
cp src/index.html dist
cp src/*.ttf dist

