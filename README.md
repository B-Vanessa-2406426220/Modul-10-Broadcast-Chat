# Reflection

## Original code of broadcast chat

![Server Result](Server.png)
![Client-1 Result](Client-1.png)
![Client-2 Result](Client-2.png)
![Client-3 Result](Client-3.png)

- Koneksi Awal: Saat setiap klien dijalankan, mereka melakukan handshake WebSocket ke alamat 127.0.0.1:2000 yang dikelola oleh server. Server akan mencetak log koneksi baru yang menyertakan alamat IP dan port unik dari setiap klien.  
- Mekanisme Pengiriman Pesan: Ketika kita mengetik teks di salah satu klien, pesan tersebut dikirim melalui stream WebSocket ke server.  
- Proses Broadcast: Server menerima pesan tersebut, lalu menggunakan broadcast channel untuk menyebarkan pesan tersebut ke seluruh klien yang sedang terhubung secara asinkronus.  
- Output pada Klien: Akibatnya, pesan yang diketik oleh satu klien akan muncul di terminal klien lainnya dengan format "From server: [isi pesan]".  
- Asynchronous Processing: Program ini menggunakan tokio::select! sehingga klien dapat terus mendengarkan pesan masuk dari server sambil menunggu input dari pengguna di terminal secara bersamaan tanpa saling memblokir (non-blocking). 
