import { useState } from "react";
import { useSearchParams } from "react-router-dom";

import { Tab, Tabs } from "@dango/shared";
import { ReceiveContainer } from "~/components/ReceiveContainer";
import { SendContainer } from "~/components/SendContainer";

const actions = ["send", "receive"];

const Transfer: React.FC = () => {
  const [searchParams, setSearchParam] = useSearchParams();
  const action = searchParams.get("action") as string;

  const [activeAction, setActiveAction] = useState<string>(
    actions.includes(action) ? action : "send",
  );

  return (
    <div className="min-h-full w-full flex-1 flex items-center justify-center z-10 relative p-4">
      <div className="flex flex-col gap-8 w-full items-center justify-center max-w-[38.5rem]">
        <div className="w-full items-center justify-end flex">
          <div>
            <Tabs
              key="transfer-actions"
              defaultSelectedKey={activeAction}
              onSelectionChange={(key) => {
                const actionKey = key.toString();
                setSearchParam({ action: actionKey });
                setActiveAction(actionKey);
              }}
              classNames={{
                tabsWrapper: "p-1 bg-surface-green-300 text-typography-green-300 rounded-2xl gap-0",
              }}
            >
              {actions.map((action) => (
                <Tab
                  key={action}
                  title={action}
                  classNames={{
                    container: "after:rounded-xl font-bold capitalize",
                    selected: "after:bg-surface-green-400 text-typography-green-400",
                  }}
                />
              ))}
            </Tabs>
          </div>
        </div>
        {activeAction === "send" ? <SendContainer /> : null}
        {activeAction === "receive" ? <ReceiveContainer /> : null}
      </div>
    </div>
  );
};

export default Transfer;
