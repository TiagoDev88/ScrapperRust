use thirtyfour::prelude::*;
use tokio;
use serde::{Serialize, Deserialize};
use serde_json::json;
use csv;
use tokio::sync::Mutex;
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
use std::error::Error;
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize)]
struct ProductData {
    url: String,
    nome: Option<String>,
    marca: Option<String>,
    sku: Option<String>,
    preco_atual: Option<f64>,
    preco_antigo: Option<f64>,
    is_purchasable: Option<bool>,
    variantes: Vec<Variant>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Variant {
    nome: String,
    is_purchasable: bool,
    preco_atual: Option<f64>,
    preco_antigo: Option<f64>,
}

async fn initialize_driver() -> WebDriverResult<WebDriver> {
    let mut caps = DesiredCapabilities::chrome();

    caps.add_arg("--disable-blink-features=AutomationControlled")?;
    caps.add_arg("--no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage")?;
    caps.add_arg("--disable-gpu")?;
    caps.add_arg("--disable-extensions")?;
    caps.add_arg("--disable-popup-blocking")?;
    caps.add_arg("--window-size=1920,1080")?;
    caps.add_arg("--disable-background-networking")?;

    caps.set_binary(r"C:\Program Files\Google\Chrome Dev\Application\chrome.exe")?; // localizacao do executavel do chrome Dev

    let driver = WebDriver::new("http://localhost:9515", caps).await?; // porta do chromedrive

    driver.execute_script(
        "Object.defineProperty(navigator, 'webdriver', {get: () => undefined})",
        Vec::<serde_json::Value>::new()
    ).await?;

    Ok(driver)
}

async fn esperar_elemento(driver: &WebDriver, seletor: &str) -> Option<WebElement> {
    for _ in 0..5 { 
        if let Ok(element) = driver.find_element(By::Css(seletor)).await {
            return Some(element);
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    None
}

async fn obter_texto(element: Option<WebElement>) -> Option<String> {
    match element {
        Some(e) => e.text().await.ok(),
        None => None,
    }
}

fn formatar_preco(preco: Option<String>) -> Option<f64> {
    preco?
        .replace("CHF", "")
        .replace(",", ".")
        .trim()
        .parse::<f64>()
        .ok()
}

async fn extrair_variantes(driver: &WebDriver) -> Vec<Variant> {
    let mut variantes = Vec::new();
    let mut variantes_processadas = HashSet::new();

    if let Ok(container) = driver.find_element(By::Css(".cs-buybox")).await {
        if let Ok(elementos_dropdown) = container.find_elements(By::Css(".cs-swatches-expand-dropdown__item")).await {
            for elemento in elementos_dropdown {
                let nome_variante = obter_texto(Some(elemento.clone())).await.unwrap_or_default();
                let class_attr = elemento.attr("class").await.unwrap_or_default().unwrap_or_default();
                let esta_disponivel = !class_attr.contains("back-in-stock-alert");

                if !nome_variante.is_empty() && !variantes_processadas.contains(&nome_variante) {
                    if let Err(e) = elemento.click().await {
                        eprintln!("Erro ao clicar na variante '{}': {:?}", nome_variante, e);
                    }
                    tokio::time::sleep(Duration::from_millis(500)).await;

                    let preco_atual = formatar_preco(obter_texto(esperar_elemento(driver, ".price-wrapper .price").await).await);
                    let preco_antigo = formatar_preco(obter_texto(esperar_elemento(driver, ".old-price .price").await).await);

                    variantes_processadas.insert(nome_variante.clone());
                    variantes.push(Variant {
                        nome: nome_variante,
                        is_purchasable: esta_disponivel,
                        preco_atual,
                        preco_antigo,
                    });
                }
            }
        }

        if let Ok(elementos_variantes) = container.find_elements(By::Css(".swatch-attribute-options .swatch-option")).await {
            for elemento in elementos_variantes {
                let nome_variante = elemento.attr("option-label").await.unwrap_or_default().unwrap_or_default();
                let class_attr = elemento.attr("class").await.unwrap_or_default().unwrap_or_default();
                let esta_disponivel = !class_attr.contains("disabled");

                if !nome_variante.is_empty() && !variantes_processadas.contains(&nome_variante) {
                    if let Err(e) = elemento.click().await {
                        eprintln!("Erro ao clicar na variante '{}': {:?}", nome_variante, e);
                    }
                    tokio::time::sleep(Duration::from_millis(500)).await; 

                    let preco_atual = formatar_preco(obter_texto(esperar_elemento(driver, ".price-wrapper .price").await).await);
                    let preco_antigo = formatar_preco(obter_texto(esperar_elemento(driver, ".old-price .price").await).await);

                    variantes_processadas.insert(nome_variante.clone());
                    variantes.push(Variant {
                        nome: nome_variante,
                        is_purchasable: esta_disponivel,
                        preco_atual,
                        preco_antigo,
                    });
                }
            }
        }
    }

    println!("✅ {} variantes extraídas.", variantes.len());
    variantes
}


async fn obter_dados_produto(driver: Arc<Mutex<WebDriver>>, url: String, handle: String, pasta_jsons: String) -> Result<(), Box<dyn Error + Send + Sync>> {
    let driver = driver.lock().await;
    println!("🔍 Acessando URL: {}", url);

    if let Err(e) = driver.get(&url).await {
        eprintln!("❌ Erro ao acessar {}: {:?}", url, e);
        return Ok(());
    }

    driver.find_element(By::Css("body")).await?;
    println!("✅ Página carregada.");

    let nome = obter_texto(esperar_elemento(&driver, ".page-title span").await).await;
    let marca = obter_texto(esperar_elemento(&driver, ".cs-buybox__brand-name span").await).await;
    let sku = obter_texto(esperar_elemento(&driver, ".cs-buybox__sku .value").await).await;
    let preco_atual = formatar_preco(obter_texto(esperar_elemento(&driver, ".price-wrapper .price").await).await);
    let preco_antigo = formatar_preco(obter_texto(esperar_elemento(&driver, ".old-price .price").await).await);
    let variantes = extrair_variantes(&driver).await;

    let dados_produto = ProductData {
        url,
        nome,
        marca,
        sku,
        preco_atual,
        preco_antigo,
        is_purchasable: Some(preco_atual.is_some()),
        variantes,
    };

    println!("📌 Dados extraídos:");
    println!("   - Nome: {:?}", dados_produto.nome);
    println!("   - Marca: {:?}", dados_produto.marca);
    println!("   - SKU: {:?}", dados_produto.sku);
    println!("   - Preço Atual: {:?}", dados_produto.preco_atual);
    println!("   - Preço Antigo: {:?}", dados_produto.preco_antigo);
    println!("   - Disponível para compra: {:?}", dados_produto.is_purchasable);

    let json_path = format!("{}/{}.json", pasta_jsons, handle);
    create_dir_all(&pasta_jsons)?;
    let json_file = File::create(&json_path)?;
    serde_json::to_writer_pretty(json_file, &dados_produto)?;

    println!("✅ JSON salvo com sucesso!");

    Ok(())
}



fn registrar_falha(handle: &str, url: &str, motivo: &str, log_falhas: &str) {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(log_falhas)
        .unwrap();
    writeln!(file, "{},{},{}", handle, url, motivo).unwrap();
}


#[derive(Debug, Deserialize)]
struct ProdutoCSV {
    Handle: String,
    #[serde(rename = "Link Update")]
    Link_Update: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let csv_input = "data/haar-shop_1_com_link_2.csv";
    let pasta_jsons = "data/haar-shop_1/json_produtos_2";

    let driver = Arc::new(Mutex::new(initialize_driver().await?));

    let mut rdr = csv::Reader::from_path(csv_input)?;
    let mut produtos_processados = HashSet::new();
    let mut handles = Vec::new();

    for result in rdr.deserialize() {
        let record: ProdutoCSV = result?;
        let handle = record.Handle;
        let url = record.Link_Update;

        if !handle.is_empty() && !url.is_empty() && !produtos_processados.contains(&handle) {
            let driver = Arc::clone(&driver);
            let pasta_jsons = pasta_jsons.clone();
            let handle_clone = handle.clone();
            let url_clone = url.clone();

            handles.push(tokio::spawn(async move {
                let _ = obter_dados_produto(driver, url_clone, handle_clone, pasta_jsons.to_string()).await;
            }));

            produtos_processados.insert(handle);
        }
    }

    futures::future::join_all(handles).await;
    println!("✅ Todos os produtos foram processados!");
    
    driver.lock().await.clone().quit().await?;
    Ok(())
}