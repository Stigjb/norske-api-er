use chrono_tz::Tz;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};
use yew::prelude::*;

use crate::components::select::Select;
use crate::fetch::{FetchError, FetchState};
use crate::gbfs::{Gbfs, SystemInformation};
use crate::link_future::LinkFuture;

const CLIENT_IDENTIFIER: &str = "stigjb-norske-api-er";

pub struct Bysykler {
    link: ComponentLink<Self>,
    system: Option<String>,
    system_info: FetchState<Gbfs<SystemInformation>>,
}

pub enum Msg {
    SystemChange(String),
    SetFetchState(FetchState<Gbfs<SystemInformation>>),
}

impl Component for Bysykler {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            system: None,
            system_info: FetchState::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SystemChange(area) => {
                self.system = Some(area);
                let system = self.system.clone().unwrap_or("yo".into());

                let future = async move {
                    match fetch_system_info(&system).await {
                        Ok(sys_info) => Msg::SetFetchState(FetchState::Success(sys_info)),
                        Err(err) => Msg::SetFetchState(FetchState::Failed(err)),
                    }
                };
                self.link.send_future(future);
                self.link
                    .send_message(Msg::SetFetchState(FetchState::Fetching));
            }
            Msg::SetFetchState(fetch) => {
                self.system_info = fetch;
            }
        };
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn rendered(&mut self, first_render: bool) {
        if !first_render || self.system.is_none() {
            return;
        }

        let system = self.system.clone().unwrap();

        let future = async move {
            match fetch_system_info(&system).await {
                Ok(sys_info) => Msg::SetFetchState(FetchState::Success(sys_info)),
                Err(err) => Msg::SetFetchState(FetchState::Failed(err)),
            }
        };
        self.link.send_future(future);
        self.link
            .send_message(Msg::SetFetchState(FetchState::Fetching));
    }

    fn view(&self) -> Html {
        let text = match &self.system_info {
            FetchState::NotFetching => html! {},
            FetchState::Fetching => html! { "Henter data ..." },
            FetchState::Success(system) => system_view(system),
            FetchState::Failed(err) => html! { format!("Failed!\n{:#?}", err) },
        };
        let onchange = self.link.callback(Msg::SystemChange);
        let options: Vec<String> = vec![
            "oslobysykkel.no".into(),
            "bergenbysykkel.no".into(),
            "trondheimbysykkel.no".into(),
            "edinburghcyclehire.com".into(),
            "oslovintersykkel.no".into(),
        ];
        html! {
            <>
                <h1>{ "Bysykler" }</h1>
                <div>
                    <form>
                        <div class="form-group">
                            <label for="system-select">{ "Velg et bysykkelsystem: " }</label>
                            <Select<String> id="system-select" class="form-control"
                                on_change=onchange options=options selected=&self.system />
                        </div>
                    </form>
                    <p>{ text }</p>
                    <p>
                        { "Bysykkeldata leveres av Urban Sharing. For informasjon om API-et, se " }
                        <a href="https://oslobysykkel.no/apne-data/sanntid">
                            { "deres hjemmesider" }
                        </a>
                        { ". Dataene er gjort tilgjengelig under " }
                        <a href="https://data.norge.no/nlod/no/2.0">
                            { "Norsk lisens for offentlige data (NLOD) 2.0" }
                        </a>
                        { "." }
                    </p>
                </div>
            </>
        }
    }
}

fn system_view(system: &Gbfs<SystemInformation>) -> Html {
    let tz: Tz = system.data.timezone.parse().unwrap_or(Tz::UTC);
    html! {
        <>
        <h2>{ "Systeminformasjon" }</h2>
        <div class="list-group">
            <div class="list-group-item">{ format!("System-ID: {}", system.data.system_id) }</div>
            <div class="list-group-item">{ format!("Språk: {}", system.data.language) }</div>
            <div class="list-group-item">{ format!("Navn: {}", system.data.name) }</div>
            <div class="list-group-item">{ format!("Operatør: {}", system.data.operator) }</div>
            <div class="list-group-item">{ format!("Tidssone: {}", system.data.timezone) }</div>
            <a href=format!("tel:{}", system.data.phone_number)
                    class="list-group-item list-group-item-action">
                { format!("Telefonnummer: {}", system.data.phone_number) }
            </a>
            <a href=format!("mailto:{}", system.data.email)
                    class="list-group-item list-group-item-action">
                { format!("E-post: {}", system.data.email) }
            </a>
            <div class="list-group-item">
                { format!("Siste status: {}", system.last_updated.with_timezone(&tz)) }
            </div>
        </div>
        </>
    }
}

async fn fetch_system_info(system: &str) -> Result<Gbfs<SystemInformation>, FetchError> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let url = format!(
        "https://gbfs.urbansharing.com/{}/system_information.json",
        system
    );
    let request: Request = Request::new_with_str_and_init(&url, &opts)?;
    request.headers().set("Accept", "application/json")?;
    // It seems that Urban Sharing's server doesn't send an appropriate
    // Access-Control-Allow-Headers in their preflight response
    request
        .headers()
        .set("Client-Identifier", CLIENT_IDENTIFIER)?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    let json: JsValue = JsFuture::from(resp.json()?).await?;
    let system_info: Gbfs<SystemInformation> = json.into_serde()?;

    Ok(system_info)
}
