const path = require("path");
const MiniCssExtractPlugin = require("mini-css-extract-plugin");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const webpack = require("webpack");

module.exports = (env) => {
  return {
    mode: "development", // 设置为 'production' 以启用压缩
    entry: {
      index: "./src/index.js",
    },
    output: {
      filename: "[name].[contenthash].bundle.js",
      path: path.resolve(__dirname, "dist"),
    },
    module: {
      rules: [
        {
          test: /\.js$/,
          exclude: /node_modules/,
          use: {
            loader: "babel-loader",
            options: {
              presets: ["@babel/preset-react", "@babel/preset-env"],
            },
          },
        },
        {
          test: /\.css$/,
          use: [MiniCssExtractPlugin.loader, "css-loader"],
        },
        {
          test: /\.(png|jpe?g|gif|svg)$/i,
          type: "asset/resource",
          generator: {
            filename: "[path][name].[ext]",
          },
        },
      ],
    },
    resolve: {
      extensions: [".js", ".jsx", ".css"],
    },
    optimization: {
      splitChunks: {
        chunks: "all",
      },
    },
    plugins: [
      new webpack.DefinePlugin({
        // 定义 process.env
        "process.env": JSON.stringify({ ...process.env }),
      }),
      new HtmlWebpackPlugin({
        template: "./public/index.html", // 指定模板文件
        filename: "index.html", // 输出文件名
      }),
      new MiniCssExtractPlugin({
        filename: "[name].[contenthash].css",
      }),
    ],
    devServer: {
      compress: true,
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
          context: ["/api"],
          target: env && env.API_URL,
          secure: false,
          changeOrigin: true,
        },
      ],
    },
  };
};
