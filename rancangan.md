contoh sederhana microservice dengan rust
- service_1 menerima request dari post
- lalu mengirimkan ke kafka agar di terima service_2
- service_2 ini menerima value dari kafka lalu membuat string hash
- lalu service_2 mengirimkan lagi ke kafka
- dan service_1 membaca value ini dari kafka
- mereturn value tersebut sebagai balasan request post
- untuk kafka saya berencana menggunakan docker-compose.yaml


- saya membutuhkan sebuah service untuk mencatat event log, seperti:
    * tracking progress
    * hitung durasi
    * dashboard
    * retry & recovery
- dan kita 
- saat service di mulai, 
