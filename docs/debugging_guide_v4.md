# R-OS v4.0.0-pre-alpha Hata Ayıklama Rehberi

Bu rehber, R-OS v4.0.0-pre-alpha bare-metal ortamında karşılaşılan karmaşık hataları sistematik olarak izlemek, yakalamak ve çözmek için gereken yöntemleri ve araç setlerini kapsar.

## 1. Hata Ayıklama Hiyerarşisi

R-OS'ta bir hata ile karşılaşıldığında izlenmesi gereken katmanlı yaklaşım:

1. **Statik Analiz (Derleme Zamanı)**: Rust derleyicisinin uyarılarını (warnings) "hata" olarak kabul edin. Çıktıdaki `unused_imports` ve `mismatched_lifetime_syntaxes` gibi uyarılar, ileride Kernel Panic olarak dönecek mantık hatalarının habercisidir.
2. **Yazılımsal İzleme (Logging)**: `dmesg` ve seri port çıktılarını inceleyin.
3. **Emülatör Denetimi (QEMU Monitor)**: CPU register ve bellek durumunu canlı olarak sorgulayın.
4. **Sembolik Hata Ayıklama (GDB)**: Kod üzerinde adım adım ilerleyerek (stepping) değişken takibi yapın.

## 2. Derleme Zamanı Analizi (Cargo Warnings)

### A. Yaşam Süresi (Lifetime) Çelişkileri
**Uyarı**: `hiding a lifetime that's elided elsewhere is confusing (sync.rs:36)`
- **Teşhis**: MutexGuard gibi yapılar, kilitledikleri veriye bağlıdır. Bu uyarıyı görmezden gelmek, çok çekirdekli (multi-core) modda "Use-After-Free" hatalarına veya beklenmedik Deadlocklara yol açabilir.
- **Çözüm**: Önerilen `spin::MutexGuard<'_, T>` sözdizimini kullanarak yaşam süresini açıkça belirtin.

### B. Atıl Kod ve Bellek Yönetimi
**Uyarı**: `unused import: alloc::string::String (memory_manager.rs:19)`
- **Teşhis**: Allocator (Slab/Buddy) henüz tam olarak devreye girmediği için String kullanımı pasif kalmış.
- **Çözüm**: Kullanılmayan importları temizleyerek bellek haritasını sadeleştirin.

### C. Çoklu Mimari (Multi-Arch) Uyarıları
**Uyarı**: `riscv64/trap.rs` içindeki gereksiz unsafe blokları.
- **Teşhis**: Bir mimaride çalışan kodun diğerinde unsafe gerektirmemesi, mimari izolasyonun (HAL) tam oturmadığını gösterebilir.

## 3. Loglama ve İzleme (kernel/libkern)

`libkern/dmesg.rs` altındaki yapı, kernel halka tamponuna (ring buffer) veri yazar.
- **Kritik Hatalar**: `kernel_log!("CRITICAL", "Mesaj")`
- **İzleme**: `kernel_log!("DEBUG", "Değişken: {:?}", var)`

## 4. QEMU Monitörü ile Canlı Analiz

Komut | Açıklama | Analiz Alanı
:--- | :--- | :---
`info registers` | CPU register durumlarını döker. | RAX, RIP, RSP takibi.
`info mem` | Aktif sayfa tablolarını ve izinlerini gösterir. | PML4 / Adres haritalama.
`xp /32gx 0x1000` | Fiziksel adresteki veriyi hex olarak döker. | Ham bellek içeriği.
`info lapic` | Local APIC durumunu gösterir. | Kesme (Interrupt) sorunları.

## 5. GDB ile Sembolik Debugging

GDB, kernel binary içindeki sembolleri okuyarak ham adresleri fonksiyon isimlerine dönüştürür.
1. QEMU'yu bekletme modunda başlatın: `cargo run -- --gdb`
2. GDB'yi çalıştırın: `gdb ./target/x86_64-unknown-none/debug/rian_cekirdek`
3. Komut satırında: `target remote :1234`

## 6. Kritik Hata Senaryoları

- **Senaryo A: Triple Fault**: `arch/x86_64/gdt.rs` içinde TSS ve Stack adresi kontrol edilmelidir.
- **Senaryo B: Deadlock**: `sync.rs` üzerindeki kilitlenme uyarılarını ciddiye alın. GDB ile `thread apply all bt` komutunu kullanın.
- **Senaryo C: Page Fault (0x0E)**: `CR2` register'ı hatalı adresi verir. memory_manager uyarıları temizlenmelidir.

## 7. Bellek Güvenliği (Poisoning)

- **Poisoning**: Tahsis edilen her bloğu `0xAA`, serbest bırakılanı `0x55` ile doldurun.
- **Multi-Core**: `multi_core` aktifken Race Condition ihtimalini her zaman listenin başına koyun.
