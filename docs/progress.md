# R-OS Progress & Roadmap Tracker (v4.0.0-pre-alpha)

## ✅ Completed Pre-Alpha Epics
- [x] **Base Architecture**: 5-domain split (`nexus`, `subsys`, `hal`, `io`, `api`) tam izole edildi.
- [x] **Unix Philosophy Migration**: "Apple Darwin" kompleks IPC sınıfları silinip "Her şey bir dosyadır" Unix mantalitesine kod bazlı geçildi.
- [x] **1. Syscalls Katmanı**: `SYS_READ`, `SYS_WRITE`, `SYS_OPEN`, `SYS_EXIT` dispatcher'ı `/src/nexus/syscall/` altına inşa edildi.
- [x] **2. Gelişmiş Unix Sinyalleri**: `SIGINT`, `SIGKILL`, `SIGSEGV` trap frameleri manipüle ederek `trap.rs` altına implement edildi.
- [x] **3. Unix Domain Sockets**: `AF_UNIX` tabanlı, Ready, Listening, Connected durumlarına (state-machine) sahip sanal soketler Vnode tipine eşlendi.
- [x] **4. RAMFS (In-Memory Filesystem)**: 10MB boyut kotalı RAM belleğinde saklanan `/tmp` modülü kernel'a kazandırıldı.
- [x] **5. Kabuk (Shell) Yönlendirmeleri**: `|`, `>`, `<` pipe/redirect stream parserları yazıldı, devasa monolithic shell kodları `builtins.rs` alt modüllerine temizlendi.
- [x] **6. Güvenlik Maskeleri (Capabilities)**: Cihaz dosyalarına (`/dev/kmem`) özel erişim yetkisi (`CAP_SYS_RAWIO`) getirilerek Security Subsystem modülüne bağlandı.
- [x] **7. VirtIO-GPU Simülasyonu**: Erken boot panic (dmesg) logları için, izole bir paralel `Mutex` içerisinde 1024x768 framebuffer (pixels) ayrılarak grafik tabanı atıldı.
- [x] **8. Gerçek Page Table Mapping (PML4)**: `copy_from_user` modülleri donanımsal mantıkla map eden bellek işaretçisi doğrulama kodlarına (map_page/unmap_page) çevrildi.
- [x] **9. Plan 9 (9P) Filesystem Network Protokolü**: Kök sanal dosya sistemi için RPC tabanlı Plan 9 (9P2000) İstemci, Codec ve Session altyapısı eklendi.
- [x] **10. Kqueue / EPoll Edge-Triggered Mekanizma**: VFS pipe okumalarındaki anlık IO polling mekanizmaları asenkron notification sistemlerinin kqueue kopyası olan `events.rs` modülüyle harmanlanıp eklendi.

---

## 🚀 Next Steps (Evre 2: Gerçek İşletim Sistemi Dinamikleri)

### 1. Kullanıcı Modu (Ring 3) Geçişi (TAMAMLANDI)
- [x] **no_std & Bare-Metal Migration**: Kernel tamamen `no_std` bağımlılığından arındırıldı ve `x86_64-unknown-none` hedefinde çalışır hale getirildi.
- [x] **GDT & TSS Yapılandırması**: Kernel ve User modları için segment tanımları yapıldı, donanımsal TSS ve Ring 0 stack switching mekanizması kuruldu.
- [x] **User-Stack Tahsisi**: Her süreç için 16KB izole edilmiş, `USER`/`WRITE`/`NX` izinli kullanıcı yığını hazırlama modülü (`src/subsys/process/stack.rs`) eklendi.
- [x] **Dynamic HAL (Multi-Arch)**: `x86_64`, `AArch64` ve `RISC-V 64` mimarileri için ortak bir `ArchContext` trait'i üzerinden soyutlama katmanı oluşturuldu.

### 2. Süreç (Process) Yaşam Döngüsü
- [x] **Süreç Hazırlığı**: C ABI (System V AMD64) uyumlu yığın hazırlama (`argc`, `argv`, `envp`) implementasyonu bitti.
- [x] **fork() Mekanizması**: Mevcut bir task'ın sayfa tablolarını (PML4) kopyalayarak (COW destekli) yeni bir süreç oluşturma.
- [x] **execve() (Mach-O Loader)**: 9P üzerinden gelen bir `.macho` dosyasını (`W^X` ve `__PAGEZERO` korumalı) belleğe map edip giriş noktasına atlama.
- [x] **Scheduler (Zamanlayıcı)**: Round-robin algoritması ile CPU zamanını süreçler arasında paylaştırma (Preemptive & Context Switching).

### 3. Donanım Genişlemesi
- [x] **VirtIO-Input**: PCI Device ID 18 taraması, lock-free ring buffer ve kqueue (Edge-triggered) entegrasyonu tamamlandı.
- [x] **PCI Bus Probing**: Sistemdeki tüm aygıtları otomatik tanıyacak bir tarama mekanizması.
- [x] **Advanced Interrupt Controller (APIC/IOAPIC)**: Modern kesme yönetimi ve I/O APIC yönlendirme tabloları.
- [x] **Frame Buffer Terminal**: Gerçek bir tty terminal emülatörü (kaydırılabilir, font yüklenebilir).

### 4. Bellek ve Sistem İyileştirmeleri (Orta Vadeli)
- [ ] **Slab/Buddy Allocator**: Bellek parçalanmasını (fragmentation) önleyen gelişmiş heap yönetimi.
- [ ] **Symmetric Multiprocessing (SMP)**: ACPI üzerinden diğer işlemci çekirdeklerini (APs) uyandırma ve eşzamanlı çalışma.
- [ ] **Disk Desteği (VFS + Fat32/Ext2)**: Kalıcı depolama birimleri için dosya sistemi sürücüleri.

### 5. Ağ ve Grafik Katmanları (Hardcore Hedefler)
- [ ] **VirtIO-Net & TCP/IP Stack**: Donanımdan bağımsız ağ sürücüsü ve temel protokoller (ARP, IP, UDP, ICMP).
- [ ] **Pencere Yöneticisi (GUI)**: Fare etkileşimli, minimal bir kompozit pencere sistemi.
- [ ] **R-Libc**: C programlarını R-OS üzerinde çalıştırmak için standart kütüphane implementasyonu.
- [ ] **Userland Toolset**: Yerleşik kabuk komutlarının (`ls`, `cat`, `rm`) kullanıcı alanında (Ring 3) çalışan sürümleri.
