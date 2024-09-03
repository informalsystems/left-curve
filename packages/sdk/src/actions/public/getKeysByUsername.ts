import type {
  Address,
  Chain,
  Client,
  Hex,
  Key,
  KeyHash,
  Signer,
  Transport,
  Username,
} from "@leftcurve/types";
import { getAppConfig } from "./getAppConfig";
import { queryWasmSmart } from "./queryWasmSmart";

export type GetKeysByUsernameParameters = {
  username: Username;
  startAfter?: Hex;
  limit?: number;
  height?: number;
};

export type GetKeysByUsernameReturnType = Promise<Record<KeyHash, Key>>;

/**
 * Get the keys associated with a username.
 * @param parameters
 * @param parameters.username The username to get keys for.
 * @param parameters.startAfter The key hash to start after.
 * @param parameters.limit The maximum number of keys to return.
 * @param parameters.height The height at which to get the keys.
 * @returns The keys associated with the username.
 */
export async function getKeysByUsername<
  chain extends Chain | undefined,
  signer extends Signer | undefined,
>(
  client: Client<Transport, chain, signer>,
  parameters: GetKeysByUsernameParameters,
): GetKeysByUsernameReturnType {
  const { username, startAfter, limit, height = 0 } = parameters;
  const msg = { keys: { username, startAfter, limit } };

  const accountFactory = await getAppConfig<Address>(client, { key: "account_factory" });

  return await queryWasmSmart(client, { contract: accountFactory, msg, height });
}
