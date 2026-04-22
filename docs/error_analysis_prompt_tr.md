# R-OS Hata Analizi ve Çözüm Rehberi (v4.0.0)

Bu doküman, R-OS mikroçekirdek geliştirme sürecinde karşılaşılan tipik donanım ve yazılım hatalarının (pathologies) teşhis ve çözüm yöntemlerini içerir.

## 1. Bellek Yönetimi Hataları

### A. Page Fault (0x0E)
**Belirti**: Çekirdek bir sanal adrese erişmeye çalıştığında sistem durur.
**Analiz**:
- `CR2` register'ını kontrol ederek hangi sanal adresin ihlal edildiğini bulun.
- `kernel/memory/paging.rs` içindeki `map_page` fonksiyonunun ilgili adresi fiziksel bir frame ile eşleyip eşlemediğini doğrulayın.
- **Çözüm**: Sayfa tablolarının (PML4) doğru kurulduğundan ve erişim izinlerinin (Read/Write/User) uygun olduğundan emin olun.

### B. Alloc Error (Heap Management)
**Belirti**: Bellek tahsisi sırasında çekirdek paniği oluşması.
**Analiz**:
- `SlabAllocator` veya `BuddyAllocator` üzerinde parçalanma (fragmentation) olup olmadığını kontrol edin.
- **Çözüm**: Sık kullanılan nesneler için `SlabCache` boyutlarını optimize edin ve `kernel/memory/allocator/buddy.rs` içindeki `coalesce` mantığını doğrulayın.

---

## 2. Eşzamanlılık ve SMP Hataları

### A. Interrupt-Safe Deadlock (Kritik)
**Belirti**: Bir kesme (interrupt) oluştuğunda sistemin tamamen kilitlenmesi.
**Kök Neden**: `spin::Mutex` kullanırken kesmelerin açık bırakılması. Bir thread kilit tutarken kesme oluşur ve ISR aynı kilidi almaya çalışırsa sistem kilitlenir.
**Çözüm**:
- Tüm global allocator ve scheduler kilitleri `interrupts::without_interrupts` bloğu içinde alınmalıdır.
```rust
// Güvenli kilit kullanımı
crate::arch::cpu::instructions::without_interrupts(|| {
    let mut allocator = ALLOCATOR.lock();
    // ...
});
```

### B. SMP Race Condition
**Belirti**: Çok çekirdekli modda verilerin tutarsız olması.
**Çözüm**: Paylaşılan statik veriler için her zaman `Atomic` tipler veya `Mutex` koruması kullanın.

---

## 3. Donanım ve Sürücü Hataları

### A. APIC Hataları
**Belirti**: Klavye veya mouse kesmelerinin gelmemesi.
**Analiz**: `arch/interrupts/apic/mod.rs` içindeki IOAPIC yönlendirme tablosu (Redirection Table) indekslerini kontrol edin.

### B. VirtIO MMIO Çakışmaları
**Belirti**: GPU veya Ağ kartının yanıt vermemesi.
**Analiz**: PCI taraması sırasında `BAR` (Base Address Register) adreslerinin bellek haritasındaki boş alanlarla çakışmadığından emin olun.

---

## 4. Debugging Araçları

R-OS içinde hata ayıklama için şu araçları kullanabilirsiniz:
1. **dmesg**: `libkern/dmesg.rs` üzerinden sistem loglarını inceleyin.
2. **Panic Handler**: `main.rs` içindeki `panic` fonksiyonunu özelleştirerek yığın izi (stack trace) yazdırın.
3. **Memory Poisoning**: Debug modunda tahsis edilen belleği `0xAA`, serbest bırakılanı `0x55` ile doldurarak "use-after-free" hatalarını yakalayın.
