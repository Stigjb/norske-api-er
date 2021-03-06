use yew::prelude::*;
use yew_router::switch::Permissive;
use yew_router::{prelude::*, route::Route};

use crate::components::nav::Nav;
use crate::routes::{bysykler::Bysykler, home::Home, luftkvalitet::Luftkvalitet, AppRoute};

/// Root component
pub struct App;

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        App {}
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="container">
                <Router<AppRoute, ()>
                    render = Router::render(|switch: AppRoute | {
                        let page = match &switch {
                            AppRoute::Home => html!{ <Home /> },
                            AppRoute::Bysykler => html!{ <Bysykler /> },
                            AppRoute::Luftkvalitet => html!{ <Luftkvalitet /> },
                            AppRoute::PageNotFound(Permissive(None)) => html!{"Page not found"},
                            AppRoute::PageNotFound(Permissive(Some(missed_route))) => {
                                html!{format!("Page '{}' not found", missed_route)}
                            },
                        };
                        html! {
                            <>
                                <Nav switch=switch />
                                { page }
                            </>
                        }
                    } )
                    redirect = Router::redirect(|route: Route<()>| {
                        AppRoute::PageNotFound(Permissive(Some(route.route)))
                    })
                />
            </div>
        }
    }
}
