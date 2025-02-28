# 🚀 Rust Web Scraper com Thirtyfour

Este projeto é um **web scraper** desenvolvido em **Rust** usando a biblioteca [`thirtyfour`](https://crates.io/crates/thirtyfour) para **automatizar a extração de dados de produtos** de um site protegido contra bots.  
O scraper utiliza **WebDriver** para navegar no site, coletar informações sobre produtos e salvar os dados em arquivos **JSON**.

## 🛠️ Tecnologias Utilizadas

- **Rust** 🦀
- [`thirtyfour`](https://crates.io/crates/thirtyfour) – Controle do WebDriver (Chrome)
- [`tokio`](https://crates.io/crates/tokio) – Execução assíncrona
- [`serde`](https://crates.io/crates/serde) e [`serde_json`](https://crates.io/crates/serde_json) – Serialização de dados
- [`csv`](https://crates.io/crates/csv) – Manipulação de arquivos CSV

## 📦 Instalação

### Pré-requisitos

1. **Instale o Rust**  
   Se ainda não tem o Rust instalado, utilize o [Rustup](https://rustup.rs/):  
   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

2. **Instale o ChromeDriver**
  O WebDriver precisa estar instalado para o Chrome. Faça o download da versão compatível com o seu navegador no site oficial:
[ChromeDriver](https://googlechromelabs.github.io/chrome-for-testing/):
  
4. **Adicione o ChromeDriver ao PATH**
3.  **No Windows (PowerShell):**
      ```powershell
     $env:Path += ";C:\caminho\para\chromedriver.exe"
3. **No Linux/MacOS:**
    ```linux
    export PATH=$PATH:/caminho/para/chromedriver
4. **git clone**
   ```git clone
     git clone https://github.com/seu-usuario/seu-repositorio.git
5. **🚀 Como usar**
   ```
    Prepare o arquivo CSV de entrada
    O scraper lê um arquivo CSV contendo os produtos a serem processados. O formato esperado é:

    Handle,Link Update
    produto-1,https://www.site.com/produto-1
    produto-2,https://www.site.com/produto-2
    Execute o scraper

6. **Execute o scraper**
    ```cargo run
    cargo run
    
7. **Os resultados serão salvos como JSON no diretório data/json_produtos/.**
 
   ```exemplo
       EXEMPLO:
     {
        "url": "https://www.site.com/produto-1",
        "nome": "Produto Exemplo",
        "marca": "Marca Exemplo",
        "sku": "12345",
        "preco_atual": 29.99,
        "preco_antigo": 34.99,
        "is_purchasable": true,
        "variantes": [
        {
          "nome": "Vermelho",
          "is_purchasable": true,
          "preco_atual": 29.99,
          "preco_antigo": 34.99
        },
        {
          "nome": "Azul",
          "is_purchasable": false,
          "preco_atual": 29.99,
          "preco_antigo": 34.99
        }
      ]
    }

