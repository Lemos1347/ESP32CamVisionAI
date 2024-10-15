set allow-duplicate-recipes

[private]
default:
  @just --list

[group("embedded")]
esp32cam:
  @just  --justfile ./embedded/Justfile flash

[group("api")]
[linux]
api:
  @(while ! nc -z localhost 8080; do sleep 1; done && xdg-open ./frontend/video.html) &
  @just --justfile ./api/Justfile run

[group("api")]
[macos]
api:
  @(while ! nc -z localhost 8080; do sleep 1; done && open ./frontend/video.html) &
  @just --justfile ./api/Justfile run

[group("api")]
[windows]
api:
  @powershell -Command "while (!(Test-NetConnection -ComputerName localhost -Port 8080).TcpTestSucceeded) { Start-Sleep 1 }; Start-Process 'frontend\\video.html'" &
  @just --justfile ./api/Justfile run
