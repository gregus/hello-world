import type { NextConfig } from "next";
import CopyWebpackPlugin from "copy-webpack-plugin";
import path from "path";

const nextConfig: NextConfig = {
  webpack: (config, { isServer }) => {
    if (!isServer) {
      // Cesium configuration for client-side
      config.plugins.push(
        new CopyWebpackPlugin({
          patterns: [
            {
              from: path.join(
                path.dirname(require.resolve("cesium")),
                "../Build/Cesium/Workers"
              ),
              to: "static/cesium/Workers",
            },
            {
              from: path.join(
                path.dirname(require.resolve("cesium")),
                "../Build/Cesium/ThirdParty"
              ),
              to: "static/cesium/ThirdParty",
            },
            {
              from: path.join(
                path.dirname(require.resolve("cesium")),
                "../Build/Cesium/Assets"
              ),
              to: "static/cesium/Assets",
            },
            {
              from: path.join(
                path.dirname(require.resolve("cesium")),
                "../Build/Cesium/Widgets"
              ),
              to: "static/cesium/Widgets",
            },
          ],
        })
      );

      // Handle Cesium's use of AMD
      config.module = config.module || {};
      config.module.unknownContextCritical = false;
      config.module.unknownContextRegExp =
        /\/cesium\/cesium\/Source\/Core\/buildModuleUrl\.js/;
    }

    return config;
  },
};

export default nextConfig;
