# Apelles Android — P2 Advanced Tools

## نتيجة التدقيق

هذه المرحلة ليست مجرد إضافة واجهة Android؛ معظم الأدوات المتقدمة تعيش في محرك Rust المشترك. لذلك يجب عدم إضافة JNI خاص لكل أداة. Android يستفيد من أوامر المحرك المشتركة، بينما يقتصر عمله على الذاكرة، اللمس، التخزين، واختيار مسار تنفيذ مناسب للجهاز.

## الموجود حاليًا

| المجال | الحالة الحالية | الملاحظة |
|---|---|---|
| Remove/Healing | موجود جزئيًا/عمليًا | توجد أدوات Remove وHealing وContent-Aware Fill محلية، مع أعمال خلفية وإلغاء؛ يلزم تحسين Preview وقياس Android |
| Puppet Warp | موجود في المحرك | يوجد `edit.puppetWarp` ونظام pins وmesh؛ الواجهة والتفاعل اللمسي يحتاجان تحسينًا |
| Perspective | موجود جزئيًا | توجد Perspective Warp وVanishing Point و`algo/perspective.rs`؛ Corner Skew/التدفق الاحترافي الكامل غير مكتمل |
| Smart Filters | موجودة | الفلاتر الحية وPSD round-trip موجودان، لكن قناع Smart Filter قابل للرسم ما زال فجوة موثقة |
| RAW | قراءة أولية | DNG/CR2/NEF/ARW تُعرّف كـ Camera Raw في codec؛ يلزم فك/تطوير RAW كامل واختبار كاميرات حقيقية |
| Camera Raw scopes | موجودة جزئيًا | Histogram وWaveform/Vectorscope ومسار Camera Raw موجودة في UI، ويجب تحسين الأداء على Android |
| ICC | موجود | توجد ملفات ICC مضمّنة وإدارة CMYK ومسارات Color Management؛ Soft Proof وواجهات العرض تحتاج إكمالًا |
| CMYK/Lab | موجودان في المحرك | توجد نماذج الألوان والتحويلات واختبارات، لكن لوحة التحكم الكاملة غير مكتملة |
| ONNX background removal | غير موجود | سجل المشروع يذكر صراحة عدم وجود ONNX runtime dependency |
| Plugins | sandbox WASM موجود | ABI v1 وقيود ذاكرة/وقت وعزل wasmi موجودة؛ التوقيع والتحقق من المصدر غير مكتملين |

## ما لم يُنفذ في هذه المرحلة

لم تتم إضافة ONNX runtime أو نموذج segmentation أو GPU/NNAPI delegate؛ لا يوجد نموذج مرخّص ومضمّن في المشروع، وإضافة نموذج كبير إلى APK دون ميزانية ذاكرة واختبارات أجهزة قد تسبب تضخم الحزمة أو OOM.

لم تتم إضافة Smart Filter Mask Painting؛ هذا يتطلب امتدادًا في نموذج المستند وPSD `filterFX`، وربطًا واضحًا بالفرشاة والتراجع والحفظ والاستعادة.

لم تتم إعادة كتابة Puppet Warp أو Perspective؛ الأساس موجود في المحرك، لكن تحسينهما يحتاج اختبارات بصرية وقياسات تفاعل لمس على جهاز فعلي.

لم تتم إضافة توقيع Plugins؛ التشغيل الحالي WASM sandboxed ومقيد، لكن توقيع الحزم والتحقق من publisher يحتاج تصميم manifest ومخزن مفاتيح وسياسة ثقة.

## خطة التنفيذ الآمنة التالية

### P2-A — أدوات محلية بلا نماذج خارجية

1. إضافة Preview منخفض الدقة لـ Remove/Healing، ثم تطبيق النتيجة على الدقة الأصلية عند التأكيد.
2. تحسين نقاط Puppet Warp اللمسية، مع شبكة مرئية، snap اختياري، وتراجع واحد للعملية.
3. إكمال Corner Skew/Homography في transform model مع اختبار round-trip في PCraft وPSD عند الإمكان.
4. إضافة قناع raster مستقل لكل Smart Filter، مع فرشاة/ممحاة، undo، وحفظ native أولًا.

### P2-B — RAW والألوان

1. فصل مرحلة decode وdevelop في Camera Raw حتى لا تُحمّل الصورة الكاملة قبل اختيار المعاينة.
2. استخدام proxy 1600–2400px للـ scopes والمعاينة.
3. اختبار ملفات DNG وCR2 وNEF وARW حقيقية من عدة كاميرات.
4. إكمال Soft Proof UI واختيار profile وrendering intent، مع اختبارات ICC وCMYK/Lab.

### P2-C — إزالة الخلفية المحلية

1. اختيار نموذج segmentation مرخّص بوضوح، مع نسخة صغيرة Android أولًا.
2. دمج inference خلف feature flag منفصل، لا في المسار الأساسي.
3. تشغيله على worker thread مع cancellation وذاكرة محدودة.
4. تجربة CPU أولًا، ثم NNAPI/GPU delegate اختياري بعد قياس التوافق.
5. إبقاء النموذج خارج APK الأساسي أو تنزيله/استيراده بموافقة واضحة إذا تجاوز حجم الحزمة المقبول.

### P2-D — Plugins موثوقة

1. الإبقاء على WASM sandbox وعدم السماح بـ native shared libraries داخل Android.
2. إضافة digest SHA-256 إلى manifest.
3. التحقق الاختياري من توقيع Ed25519 قبل التثبيت.
4. عرض المصدر والصلاحيات والحدود للمستخدم.
5. إضافة ABI v2 فقط بعد تحديد backward compatibility مع ABI v1.

## الاختبارات المطلوبة

- `cargo test -p apelles-algo` لاختبارات warp/puppet/segmentation.
- `cargo test -p apelles-engine smart_psd` لاختبارات Smart Filters.
- `cargo test -p apelles-codecs --test camera_raw` وملفات corpus حقيقية.
- اختبارات ICC/CMYK/Lab في `apelles-color`, `apelles-raster`, وPSD.
- اختبارات WASM ABI والميزانية في `apelles-plugins`.
- Android device tests على أجهزة 4 و6 و8 GB RAM، مع Vulkan وOpenGL، وإلغاء كل عملية أثناء التنفيذ.

لم تُشغّل هذه الأوامر في البيئة الحالية لأن `cargo` وGradle غير مثبتين، ولم يتوفر جهاز Android حقيقي.
