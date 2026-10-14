# Apelles على Android

تم تجهيز Apelles للعمل على Android عبر نسخة WebAssembly قابلة للتثبيت كتطبيق **PWA**. هذا المسار يعيد استخدام محرك Rust وواجهة المستخدم نفسها، مع تحسينات مناسبة للشاشات الصغيرة.

## بناء النسخة

من جذر المستودع:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
cd apps/apelles-web
trunk build --release
```

سيتم إنشاء الملفات داخل `dist/web/`.

## تثبيت التطبيق على Android

1. ارفع محتويات مجلد `dist/web/` إلى خادم HTTPS.
2. افتح الرابط في Chrome على Android.
3. اختر **Install app** أو **Add to Home screen**.
4. سيعمل Apelles كـ PWA مستقل مع واجهة خاصة به.

## ملاحظات مهمة

- أول تشغيل يحتاج اتصالاً لتحميل ملف WebAssembly الخاص بالتطبيق.
- بعد ذلك يمكن لـ Service Worker تخزين الملفات المستخدمة في الواجهة.
- فتح الملفات وحفظها يتم عبر منتقي الملفات والتنزيلات في المتصفح.
- دعم اللمس والقلم يعتمد على دعم المتصفح و`egui`، وقد يحتاج بعض الأدوات إلى تحسين خاص بالشاشات الصغيرة.
- هذه نسخة Android مبنية على الويب وليست APK أصلية. يمكن لاحقًا تغليف مجلد `dist/web/` عبر Capacitor أو Trusted Web Activity.

## التشغيل المحلي للاختبار

```bash
cargo test -p apelles-web
```

## نصائح عملية

- استخدم HTTPS في الاستضافة، لأن بعض ميزات PWA تتطلب أمان الموقع.
- عند اختبار التطبيق على الهاتف، جرّب أولاً ملف WebAssembly ثم اختبر فتح/حفظ الملفات.
- إذا واجهت مشكلة في التثبيت، تأكد أن Chrome على الهاتف يدعم PWA، وأن الموقع يعمل عبر HTTPS.

## الروابط السريعة

- مستودع المشروع: https://github.com/Paplo-painite/apelles
- إصدار Android: https://github.com/Paplo-painite/apelles/releases
- الموقع الرسمي: https://getartcraft.com/apps/apelles
