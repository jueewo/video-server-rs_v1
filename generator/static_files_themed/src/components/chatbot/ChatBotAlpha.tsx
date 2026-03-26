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
          btnclass: "btn-primary-theme",
          fkt: () => shownextcard(),
        },
        {
          id: "2",
          btntxt: "no",
          btnicon: "no-symbol",
          btnclass: "btn-secondary-theme",
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
        <div class="flex items-center p-3">
          <img src={logo_img} alt="logo" class="w-12 h-12 rounded-xl" />

          <div class="relative ml-2">
            <button
              class="p-1 rounded-md text-white/80 hover:bg-white/10 transition-colors"
              onClick={(e) => {
                const menu = (e.currentTarget as HTMLElement).nextElementSibling;
                menu?.classList.toggle("hidden");
              }}
            >
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
            </button>
            <div class="hidden absolute left-0 top-full mt-1 z-50 bg-surface shadow-lg rounded-lg p-3 border border-border w-52">
              <label class="flex items-center gap-2 text-sm text-text">
                Auto
                <input
                  type="checkbox"
                  checked={showhistorymode}
                  class="w-4 h-4 rounded border-border text-primary"
                  onChange={(e) =>
                    setShowhistorymode((e.target as HTMLInputElement).checked)
                  }
                />
                Scroll
              </label>
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
            <div class="mb-2 flex gap-3">
              <div class="flex-shrink-0">
                <div class="w-10 h-10 rounded-full overflow-hidden">
                  <img src={robot_img} alt="robot" class="w-full h-full object-cover" />
                </div>
              </div>

              <div class="flex-1">
                {/* TEXT */}
                {message.mtype === "text" && (
                  <div
                    class="inline-block bg-surface-alt text-text rounded-2xl rounded-tl-none px-4 py-2 max-w-xs"
                    dangerouslySetInnerHTML={{ __html: message.payload.text }}
                  />
                )}

                {/* CARD */}
                {message.mtype === "card" && (
                  <div class="card-theme w-52 relative overflow-hidden">
                    <figure>
                      <img src={message.payload.card.pic} alt="Card" class="w-full h-32 object-cover" />
                    </figure>
                    <div class="absolute inset-0 bg-black/40 flex flex-col justify-end p-3">
                      <h2 class="text-sm font-bold text-white">{message.payload.card.title}</h2>
                      <p class="text-xs text-white/80 mt-1">{message.payload.card.desc}</p>
                      <div class="flex justify-end mt-2">
                        <a href={message.payload.card.url}>
                          <button class="text-xs px-2 py-1 bg-primary text-white rounded-md hover:bg-primary/80 transition-colors">
                            {message.payload.card.btntxt}
                          </button>
                        </a>
                      </div>
                    </div>
                  </div>
                )}

                {/* INTERACTION */}
                {message.mtype === "interaction" && (
                  <div class="inline-block bg-surface-alt text-text rounded-2xl rounded-tl-none px-4 py-2 max-w-xs">
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
                      <div class="flex gap-2 mt-2">
                        {message.payload.interaction.choiceoptions?.map(
                          (choice: ChoiceOption) => (
                            <div key={choice.id} class="pt-1">
                              <button
                                onClick={choice.fkt}
                                class={`text-xs px-3 py-1 rounded-md transition-colors ${
                                  choice.btnclass === "btn-primary-theme"
                                    ? "bg-primary text-white hover:bg-primary/80"
                                    : "bg-secondary text-white hover:bg-secondary/80"
                                }`}
                              >
                                {choice.btntxt}
                              </button>
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
                                <span class="text-xs px-3 py-1 rounded-md bg-surface-raised text-text-muted inline-block">
                                  {choice.btntxt}
                                </span>
                              </div>
                            ),
                        )}
                      </div>
                    )}
                  </div>
                )}
              </div>
            </div>
          </div>
        ))}

        {creatingresponse && (
          <div class="flex gap-2 ml-14">
            <div class="w-2 h-2 bg-primary/60 rounded-full animate-bounce" style="animation-delay: 0ms"></div>
            <div class="w-2 h-2 bg-primary/60 rounded-full animate-bounce" style="animation-delay: 150ms"></div>
            <div class="w-2 h-2 bg-primary/60 rounded-full animate-bounce" style="animation-delay: 300ms"></div>
          </div>
        )}
      </div>
    </div>
  );
};

export default ChatBotAlpha;
