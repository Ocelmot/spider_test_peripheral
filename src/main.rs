use std::{io, path::PathBuf};

use spider_client::{
    message::{Message, UiElement, UiElementKind, UiMessage, UiPageManager, UiPath, UiInput, RouterMessage},
    AddressStrategy, Relation, Role, SpiderClient, SpiderId2048,
};

struct State {
    test_page: UiPageManager,
    page_num: usize,
    page_text: String,
}

impl State {
    async fn init(client: &mut SpiderClient) -> Self {
        let msg = RouterMessage::SetIdentityProperty("name".into(), "Test Peripheral".into());
        let msg = Message::Router(msg);
        client.send(msg).await;

        let id = client.self_relation().id;
        let mut test_page = UiPageManager::new(id, "Test Page...");
        let mut root = test_page
            .get_element_mut(&UiPath::root())
            .expect("all pages have a root");
        root.set_kind(UiElementKind::Rows);
        root.append_child(UiElement::from_string("Value is: "));
        root.append_child({
            let mut element = UiElement::from_string("0");
            element.set_id("data");
            element
        });

        root.append_child({
            let mut child = UiElement::new(UiElementKind::Columns);
            child.append_child(UiElement::from_string("Increase:"));
            child.append_child(UiElement::new(UiElementKind::Spacer));
            child.append_child({
                let mut element = UiElement::from_string("Increase");
                element.set_kind(UiElementKind::Button);
                element.set_selectable(true);
                element.set_id("button");
                element
            });
            child.append_child({
                let mut element = UiElement::from_string("Increase Five");
                element.set_kind(UiElementKind::Button);
                element.set_selectable(true);
                element.set_id("increase_5");
                element
            });
            child
        });

        
        root.append_child({
            let mut child = UiElement::new(UiElementKind::Columns);
            child.append_child(UiElement::from_string("Decrease:"));
            child.append_child(UiElement::new(UiElementKind::Spacer));
            child.append_child({
                let mut element = UiElement::from_string("Decrease");
                element.set_kind(UiElementKind::Button);
                element.set_selectable(true);
                element.set_id("decrease");
                element
            });
            child.append_child({
                let mut element = UiElement::from_string("Decrease Five");
                element.set_kind(UiElementKind::Button);
                element.set_selectable(true);
                element.set_id("decrease_5");
                element
            });
            child
        });

        root.append_child({
            let mut element = UiElement::from_string("Reset");
            element.set_kind(UiElementKind::Button);
            element.set_selectable(true);
            element.set_id("button3");
            element
        });
        
        
        
        
        root.append_child({
            let mut element = UiElement::from_string("Output:");
            element.set_id("Output");
            element
        });
        root.append_child({
            let mut element = UiElement::from_string("Text Input");
            element.set_kind(UiElementKind::TextEntry);
            element.set_selectable(true);
            element.set_id("TextInput");
            element
        });
        drop(root);

        test_page.get_changes(); // clear changes to synch, since we are going to send the whole page at first. This
                                 // Could instead set the initial elements with raw and then recalculate ids
        let msg = Message::Ui(UiMessage::SetPage(test_page.get_page().clone()));
        client.send(msg).await;

        Self {
            test_page,
            page_num: 0,
            page_text: String::new(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    println!("Hello, world!");

    let client_path = PathBuf::from("client_state.dat");
    let mut client = if client_path.exists() {
        SpiderClient::from_file(&client_path)
    } else {
        let mut client = SpiderClient::new();
        client.set_state_path(&client_path);
        client.add_strat(AddressStrategy::Addr(String::from("localhost:1930")));
        client.save();
        client
    };

    if !client.has_host_relation() {
        let path = PathBuf::from("spider_keyfile.json");

        let data = match std::fs::read_to_string(&path) {
            Ok(str) => str,
            Err(_) => String::from("[]"),
        };
        let id: SpiderId2048 = serde_json::from_str(&data).expect("Failed to deserialize spiderid");
        let host = Relation {
            id,
            role: Role::Peer,
        };
        client.set_host_relation(host);
        client.save();
    }

    client.connect().await;
    let mut state = State::init(&mut client).await;

    loop {
        match client.recv().await {
            Some(msg) => msg_handler(&mut client, &mut state, msg).await,
            None => break, //  done!
        }
    }

    Ok(())
}

async fn msg_handler(client: &mut SpiderClient, state: &mut State, msg: Message) {
    match msg {
        Message::Ui(msg) => ui_handler(client, state, msg).await,
        Message::Dataset(_) => {}
        Message::Router(_) => {}
    }
}

async fn ui_handler(client: &mut SpiderClient, state: &mut State, msg: UiMessage) {
    match msg {
        UiMessage::Subscribe => {}
        UiMessage::Pages(_) => {}
        UiMessage::GetPage(_) => {}
        UiMessage::Page(_) => {}
        UiMessage::UpdateElementsFor(_, _) => {}
        UiMessage::InputFor(_, _, _, _) => {}
        UiMessage::SetPage(_) => {}
        UiMessage::ClearPage => {}
        UiMessage::UpdateElements(_) => {}
        UiMessage::Input(element_id, _, change) => {
            
            //let mut element = element.expect("Recieved update for non existent element");

            match element_id.as_str(){
                "button" => {
                    let mut element = state.test_page.get_by_id_mut("data").unwrap();
                    state.page_num += 1;
                    element.set_text(format!("{}", state.page_num));
                },
                "increase_5" => {
                    let mut element = state.test_page.get_by_id_mut("data").unwrap();
                    state.page_num += 5;
                    element.set_text(format!("{}", state.page_num));
                },
                "decrease" => {
                    let mut element = state.test_page.get_by_id_mut("data").unwrap();
                    state.page_num = state.page_num.saturating_sub(1);
                    element.set_text(format!("{}", state.page_num));
                },
                "decrease_5" => {
                    let mut element = state.test_page.get_by_id_mut("data").unwrap();
                    state.page_num = state.page_num.saturating_sub(5);
                    element.set_text(format!("{}", state.page_num));
                },
                "button3" => {
                    let mut element = state.test_page.get_by_id_mut("data").unwrap();
                    state.page_num = 0;
                    element.set_text(format!("{}", state.page_num));
                },
                "TextInput" => {
                    if let UiInput::Text(text) = change{
                        let mut element = state.test_page.get_by_id_mut("Output").unwrap();
                        state.page_text = text;
                        element.set_text(format!("Output: {}", state.page_text));
                    }
                },
                _ => {return}
            }

            // send updates
            let changes = state.test_page.get_changes();
            let msg = Message::Ui(UiMessage::UpdateElements(changes));
            client.send(msg).await;
        }
        UiMessage::Dataset(_, _) => todo!(),
    }
}
