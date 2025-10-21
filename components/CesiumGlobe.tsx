"use client";

import { useEffect, useRef } from "react";
import * as Cesium from "cesium";

// Import Cesium CSS
import "cesium/Build/Cesium/Widgets/widgets.css";

const CesiumGlobe = () => {
  const cesiumContainer = useRef<HTMLDivElement>(null);
  const viewerRef = useRef<Cesium.Viewer | null>(null);

  useEffect(() => {
    if (!cesiumContainer.current) return;

    // Set Cesium ion access token (using the default token for development)
    // You should replace this with your own token from https://cesium.com/ion/
    Cesium.Ion.defaultAccessToken = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJqdGkiOiJlYWE1OWUxNy1mMWZiLTQzYjYtYTQ0OS1kMWFjYmFkNjc5YzYiLCJpZCI6NTc3MzMsImlhdCI6MTYyNzg0NTE4Mn0.XcKpgANiY19MC4bdFUXMVEBToBmqS8kuYpUlxJHYZxk";

    // Configure Cesium base URL for static assets
    (window as any).CESIUM_BASE_URL = "/static/cesium";

    // Create the Cesium Viewer
    const viewer = new Cesium.Viewer(cesiumContainer.current, {
      baseLayerPicker: true,
      geocoder: true,
      homeButton: true,
      sceneModePicker: true,
      navigationHelpButton: true,
      animation: false,
      timeline: false,
      fullscreenButton: true,
    });

    // Load terrain asynchronously
    Cesium.createWorldTerrainAsync().then((terrainProvider) => {
      viewer.terrainProvider = terrainProvider;
    });

    viewerRef.current = viewer;

    // Cleanup function
    return () => {
      if (viewerRef.current) {
        viewerRef.current.destroy();
        viewerRef.current = null;
      }
    };
  }, []);

  return (
    <div
      ref={cesiumContainer}
      style={{
        width: "100%",
        height: "100vh",
      }}
    />
  );
};

export default CesiumGlobe;
