const path = require("path");

module.exports = (env) => {
  return {
    target: "web",
    mode: "production",
    entry: "./src/index.js", // 入口文件
    output: {
      filename: "main.js", // 输出文件
      path: path.resolve(__dirname, "dist"), // 输出目录
    },
    module: {
      rules: [
        {
          test: /\.js$/,
          exclude: /node_modules/,
          use: {
            loader: "babel-loader",
            options: {
              presets: ["@babel/preset-env"],
            },
          },
        },
      ],
    },
    devServer: {
      compress: false,
      host: "localhost",
      client: {
        overlay: false,
      },
      port: 3000,
      historyApiFallback: true,
      static: {
        directory: path.join(__dirname, "dist"),
      },
      headers: {
        // to enable SharedArrayBuffer and ONNX multithreading
        // https://cloudblogs.microsoft.com/opensource/2021/09/02/onnx-runtime-web-running-your-machine-learning-model-in-browser/
        "Cross-Origin-Opener-Policy": "same-origin",
        "Cross-Origin-Embedder-Policy": "credentialless",
      },
      proxy: [
        {
          context: (param) =>
            param.match(
              /\/api\/.*|analytics\/.*|static\/.*|admin(?:\/(.*))?.*|profiler(?:\/(.*))?.*|documentation\/.*|django-rq(?:\/(.*))?/gm,
            ),
          target: env && env.API_URL,
          secure: false,
          changeOrigin: true,
        },
      ],
    },
  };
};
