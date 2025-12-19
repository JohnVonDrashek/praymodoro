import type { ForgeConfig } from '@electron-forge/shared-types';
import { MakerSquirrel } from '@electron-forge/maker-squirrel';
import { MakerZIP } from '@electron-forge/maker-zip';
import { MakerDeb } from '@electron-forge/maker-deb';
import { MakerRpm } from '@electron-forge/maker-rpm';
// MakerDMG removed - macOS builds now use Tauri
import { AutoUnpackNativesPlugin } from '@electron-forge/plugin-auto-unpack-natives';
import { WebpackPlugin } from '@electron-forge/plugin-webpack';
import { FusesPlugin } from '@electron-forge/plugin-fuses';
import { FuseV1Options, FuseVersion } from '@electron/fuses';

import { mainConfig } from './webpack.main.config';
import { rendererConfig } from './webpack.renderer.config';

const config: ForgeConfig = {
  packagerConfig: {
    asar: true,
    icon: './assets/icons/Praymodoro',
    executableName: 'praymodoro',
    // Code signing for macOS - requires environment variables:
    // APPLE_TEAM_ID: Your Apple Developer Team ID
    // APPLE_ID: Your Apple ID email
    // APPLE_PASSWORD: App-specific password (create at appleid.apple.com)
    ...(process.env.APPLE_TEAM_ID && {
      osxSign: {
        identity: `Developer ID Application: ${process.env.APPLE_IDENTITY || 'JOHN THOMAS VONDRASHEK'} (${process.env.APPLE_TEAM_ID})`,
        optionsForFile: () => ({
          hardenedRuntime: true,
          entitlements: './entitlements.plist',
        }),
      },
      osxNotarize: {
        appleId: process.env.APPLE_ID!,
        appleIdPassword: process.env.APPLE_PASSWORD!,
        teamId: process.env.APPLE_TEAM_ID!,
      },
    }),
  },
  rebuildConfig: {},
  makers: [
    new MakerSquirrel({
      setupIcon: './assets/icons/Praymodoro.ico',
    }),
    new MakerZIP({}, ['win32']),
    new MakerRpm({
      options: {
        icon: './assets/icons/Praymodoro.png',
      },
    }),
    new MakerDeb({
      options: {
        icon: './assets/icons/Praymodoro.iconset/icon_512x512.png',
      },
    }),
  ],
  plugins: [
    new AutoUnpackNativesPlugin({}),
    new WebpackPlugin({
      mainConfig,
      renderer: {
        config: rendererConfig,
        entryPoints: [
          {
            html: './src/index.html',
            js: './src/renderer.ts',
            name: 'main_window',
            preload: {
              js: './src/preload.ts',
            },
          },
          {
            html: './src/menu/menu.html',
            js: './src/menu/menu-renderer.ts',
            name: 'menu_window',
            preload: {
              js: './src/menu/menu-preload.ts',
            },
          },
        ],
      },
    }),
    // Fuses are used to enable/disable various Electron functionality
    // at package time, before code signing the application
    new FusesPlugin({
      version: FuseVersion.V1,
      [FuseV1Options.RunAsNode]: false,
      [FuseV1Options.EnableCookieEncryption]: true,
      [FuseV1Options.EnableNodeOptionsEnvironmentVariable]: false,
      [FuseV1Options.EnableNodeCliInspectArguments]: false,
      [FuseV1Options.EnableEmbeddedAsarIntegrityValidation]: true,
      [FuseV1Options.OnlyLoadAppFromAsar]: true,
    }),
  ],
};

export default config;
