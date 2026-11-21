# HtetWord for Android (မြန်မာစာအရေးအသား အက်ပ်)

HtetWord is designed to run seamlessly on Android smartphones and tablets in two ways:

## 1. Standalone Android APK Package
The pre-packaged Android Application package is available in:
- `dist/release/HtetWord-0.18.1-Android.apk`
- `dist/release/HtetWord-0.18.1-Android-Package.zip`

### Opening & Building in Android Studio:
1. Extract `HtetWord-0.18.1-Android-Package.zip` or open the `packaging/android` directory in **Android Studio**.
2. Android Studio will automatically recognize Gradle and the project structure.
3. Click **Build > Build Bundle(s) / APK(s) > Build APK(s)** or run:
   ```bash
   ./gradlew assembleRelease
   ```
4. Transfer the resulting `.apk` to your Android device and install.

---

## 2. Instant PWA Installation (Zero Setup & Full Offline Support)
HtetWord fully supports Progressive Web App (PWA) standards on Android:
1. Start the web server:
   ```bash
   cd apps/wordcraft-web
   trunk serve
   ```
2. Open the URL (e.g., `http://<your-ip>:8771`) in Google Chrome or Samsung Internet on your Android device.
3. Tap the browser menu (⋮) and select **"Install App"** or **"Add to Home screen"**.
4. **HtetWord** is installed directly as a native-feel application with Pyidaungsu Font, offline storage, and full-screen Liquid Glass UI!
