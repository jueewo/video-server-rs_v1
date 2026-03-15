import { useState, useEffect, useRef } from "preact/hooks";
import type { FunctionComponent } from "preact";

interface Message {
  id: string;
  mtype: "text" | "card" | "interaction";
  payload: any;
  fromUser: boolean;
  active?: boolean;
  selected?: string | null;
}

interface Card {
  title: string;
  desc: string;
  url: string;
  pic: string;
  btntxt: string;
}

interface ChoiceOption {
  id: string;
  btntxt: string;
  btnicon: string;
  btnclass: string;
  fkt: () => void;
}

interface Interaction {
  id: string;
  title: string;
  desc: string;
  itype: string;
  choiceoptions?: ChoiceOption[];
}

const ChatBotAlpha: FunctionComponent = () => {
  const chatwindow = useRef<HTMLDivElement>(null);
  const chatlog = useRef<HTMLDivElement>(null);

  const [showhistorymode, setShowhistorymode] = useState(false);
  const [messages, setMessages] = useState<Message[]>([]);
  const [randomcardindex, setRandomcardindex] = useState(0);
  const [creatingresponse, setCreatingresponse] = useState(false);

  const robot_img = "/images/chatbot/profile/profile_robot_3.jpg";
  const logo_img = "/images/chatbot/profile/logo_jwvntrs_icon.png";

  // Store cards in state so we can modify it
  const [cards, setCards] = useState<Card[]>([
    {
      title: "BPMN & Sailing seminar!",
      desc: "Learn process modeling & optimization on board of a sailing yacht.",
      url: "/en/academy/en/c-pro-sail-1",
      pic: "/images/chatbot/content/logo_bpmn_sail.jpg",
      btntxt: "more",
    },
    {
      title: "Plan4n!",
      desc: "Plan, visualize & simulate your processes.",
      url: "/en/products/en/moto",
      pic: "/images/chatbot/content/logo_plan4n_2.jpg",
      btntxt: "more",
    },
    {
      title: "Academy starting soon!",
      desc: "Our new program is being launched soon",
      url: "/en/blog/en/academy_new_program",
      pic: "/images/chatbot/content/logo_bpmn_seminar.jpg",
      btntxt: "more",
    },
    {
      title: "About me!",
      desc: "Would you like to learn more about me?",
      url: "/en/blog/en/my_new_chatbot",
      pic: "/images/chatbot/content/logo_agentforge_4.jpg",
      btntxt: "more",
    },
  ]);

  const get_card = (): Card | null => {
    const next_card_index = Math.floor(Math.random() * cards.length);
    let next_card: Card | null = null;

    if (next_card_index < cards.length) {
      next_card = JSON.parse(JSON.stringify(cards[next_card_index]));
      setCards((prevCards) => {
        const newCards = [...prevCards];
        newCards.splice(next_card_index, 1);
        return newCards;
      });
    }
    return next_card;
  };

  const shownextcard = () => {
    // console.log("SHOW NEXT");

    setMessages((prevMessages) => {
      const newMessages = [...prevMessages];
      const lastmessage = newMessages[newMessages.length - 1];

      if (lastmessage && lastmessage.mtype === "interaction") {
        lastmessage.active = false;
        lastmessage.selected = "1";
      }

      return newMessages;
    });

    const next_card = get_card();
    if (next_card) {
      const interactionMsg: Message = {
        id: "7",
        mtype: "interaction",
        payload: {
          interaction:
            interactions[Math.floor(Math.random() * interactions.length)],
        },
        fromUser: false,
        active: true,
        selected: null,
      };

      const cardMsg: Message = {
        id: "6",
        mtype: "card",
        payload: {
          card: next_card,
        },
        fromUser: false,
      };

      sendMessage(cardMsg, [interactionMsg]);
    } else {
      sendMessage({
        id: "99",
        mtype: "interaction",
        payload: {
          interaction: {
            id: "1",
            title:
              "Sorry, no more tips for now. <br>Thank you for your time! <br>I hope we talk again soon.",
            desc: "",
            itype: "info",
          },
        },
        fromUser: false,
      });

      setTimeout(() => {
        setShowhistorymode(true);
      }, 3000);
    }
  };

  const showthankyoucard = () => {
    // console.log("SHOW Thank you card");

    setMessages((prevMessages) => {
      const newMessages = [...prevMessages];
      const lastmessage = newMessages[newMessages.length - 1];

      if (lastmessage && lastmessage.mtype === "interaction") {
        lastmessage.active = false;
        lastmessage.selected = "2";
      }

      return newMessages;
    });

    sendMessage({
      id: "7",
      mtype: "text",
      payload: {
        text: "Thank you for your time! <br>I hope we talk again soon.",
      },
      fromUser: false,
    });
  };

  const interactions: Interaction[] = [
    {
      id: "1",
      title: "More Tips?",
      desc: "",
      itype: "choice",
      choiceoptions: [
        {
          id: "1",
          btntxt: "yes",
          btnicon: "check-badge",
          btnclass: "btn-primary",
          fkt: () => shownextcard(),
        },
        {
          id: "2",
          btntxt: "no",
          btnicon: "no-symbol",
          btnclass: "btn-secondary",
          fkt: () => showthankyoucard(),
        },
      ],
    },
  ];

  const [demoMessages, setDemoMessages] = useState<Message[]>([
    {
      id: "1",
      mtype: "text",
      payload: {
        text: "Welcome! <br>I'm the alpha version of a LLM-based chatbot to help you navigating our content, services and products.",
      },
      fromUser: false,
    },
    {
      id: "2",
      mtype: "text",
      payload: {
        text: "For now I'm in a closed test modus and so I'm just able to offer you some links to our recent news.",
      },
      fromUser: false,
    },
    {
      id: "3",
      mtype: "text",
      payload: {
        text: "My tip of the day:",
      },
      fromUser: false,
    },
    {
      id: "4",
      mtype: "card",
      payload: {
        card: get_card(),
      },
      fromUser: false,
    },
    {
      id: "5",
      mtype: "interaction",
      payload: {
        interaction:
          interactions[Math.floor(Math.random() * interactions.length)],
      },
      fromUser: false,
      active: true,
      selected: null,
    },
  ]);

  const sendMessage = (msg: Message, additionalMessages: Message[] = []) => {
    setCreatingresponse(true);

    setTimeout(() => {
      setCreatingresponse(false);
      setMessages((prev) => [...prev, msg]);

      if (additionalMessages.length > 0) {
        setTimeout(() => {
          sendMessage(additionalMessages[0], additionalMessages.slice(1));
        }, 600);
      } else if (demoMessages.length > 0) {
        setDemoMessages((prevDemo) => {
          const newDemo = [...prevDemo];
          const nextMsg = newDemo.shift();
          if (nextMsg) {
            setTimeout(() => {
              sendMessage(nextMsg);
            }, 600);
          }
          return newDemo;
        });
      }
    }, 600);
  };

  useEffect(() => {
    setRandomcardindex(Math.floor(Math.random() * cards.length));

    // Start the chatbot simulation
    setDemoMessages((prevDemo) => {
      const newDemo = [...prevDemo];
      const firstMsg = newDemo.shift();
      if (firstMsg) {
        sendMessage(firstMsg);
      }
      return newDemo;
    });
  }, []);

  return (
    <div ref={chatwindow} class="h-screen overflow-y-auto overscroll-contain">
      <div class="absolute top-0 z-30 bg-primary w-full">
        <div class="navbar">
          <div class="navbar-start">
            <img src={logo_img} alt="logo" class="w-12 h-12 rounded-xl" />

            <div class="dropdown dropdown-hover">
              <label tabIndex={0} class="btn btn-sm btn-ghost m-1">
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke-width="1.5"
                  stroke="currentColor"
                  class="w-6 h-6"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    d="M19.5 8.25l-7.5 7.5-7.5-7.5"
                  />
                </svg>
              </label>
              <ul
                tabIndex={0}
                class="dropdown-content z-[1] menu p-2 shadow bg-base-100 rounded-box w-52"
              >
                <label class="label">
                  Auto
                  <input
                    type="checkbox"
                    checked={showhistorymode}
                    class="toggle m-2"
                    onChange={(e) =>
                      setShowhistorymode((e.target as HTMLInputElement).checked)
                    }
                  />
                  Scroll
                </label>
              </ul>
            </div>
          </div>
        </div>
      </div>

      <div
        ref={chatlog}
        class={`max-h-fit py-20 px-3 lg:px-5 flex flex-col justify-end ${
          !showhistorymode ? "h-screen" : ""
        }`}
      >
        {messages.map((message) => (
          <div
            key={message.id}
            class={`message ${message.fromUser ? "user" : ""}`}
          >
            <div class="mb-2 chat chat-start">
              <div class="chat-image avatar">
                <div class="w-10 rounded-full">
                  <img src={robot_img} alt="robot" />
                </div>
              </div>

              {/* TEXT */}
              {message.mtype === "text" && (
                <div
                  class="chat-bubble"
                  dangerouslySetInnerHTML={{ __html: message.payload.text }}
                />
              )}

              {/* CARD */}
              {message.mtype === "card" && (
                <div class="card w-52 bg-base-100 shadow-xl image-full">
                  <figure>
                    <img src={message.payload.card.pic} alt="Sailing seminar" />
                  </figure>
                  <div class="card-body">
                    <h2 class="card-title">{message.payload.card.title}</h2>
                    <p>{message.payload.card.desc}</p>
                    <div class="card-actions justify-end">
                      <a href={message.payload.card.url}>
                        <button class="btn btn-primary btn-xs">
                          {message.payload.card.btntxt}
                        </button>
                      </a>
                    </div>
                  </div>
                </div>
              )}

              {/* INTERACTION */}
              {message.mtype === "interaction" && (
                <div class="chat-bubble">
                  <div
                    dangerouslySetInnerHTML={{
                      __html: message.payload.interaction.title,
                    }}
                  />
                  <div
                    dangerouslySetInnerHTML={{
                      __html: message.payload.interaction.desc,
                    }}
                  />
                  {message.active ? (
                    <div>
                      {message.payload.interaction.choiceoptions?.map(
                        (choice: ChoiceOption) => (
                          <div key={choice.id} class="join pt-1">
                            <div
                              onClick={choice.fkt}
                              class={`btn btn-xs px-4 ${choice.btnclass}`}
                            >
                              {choice.btntxt}
                            </div>
                          </div>
                        ),
                      )}
                    </div>
                  ) : (
                    <div>
                      {message.payload.interaction.choiceoptions?.map(
                        (choice: ChoiceOption) =>
                          choice.id === message.selected && (
                            <div key={choice.id} class="pt-1">
                              <div class="btn btn-xs px-4 btn-neutral">
                                <p>{choice.btntxt}</p>
                              </div>
                            </div>
                          ),
                      )}
                    </div>
                  )}
                </div>
              )}
            </div>
          </div>
        ))}

        {creatingresponse && (
          <div>
            <span class="loading loading-ring loading-lg"></span>
            <span class="loading loading-dots loading-lg"></span>
          </div>
        )}
      </div>
    </div>
  );
};

export default ChatBotAlpha;
