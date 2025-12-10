import { Client, GatewayIntentBits, REST, Routes } from "discord.js";
import { token, INTERVAL_MS } from "./lib/config";
import { commands, handleClearForum } from "./commands";
import { checkRssFeeds } from "./lib/rss";

const client = new Client({
  intents: [GatewayIntentBits.Guilds],
});

client.once("clientReady", async () => {
  console.log(`Logged in as: ${client.user?.tag}`);

  // Register slash commands
  try {
    const rest = new REST().setToken(token!);
    await rest.put(Routes.applicationCommands(client.user!.id), { body: commands });
    console.log("✅ Slash commands registered successfully");
  } catch (error) {
    console.error("Failed to register slash commands:", error);
  }

  // Start checking RSS feeds
  checkRssFeeds(client);
  setInterval(() => checkRssFeeds(client), INTERVAL_MS);
  console.log(`⏰ Checking RSS feeds every ${process.env.INTERVAL_MINUTES ? process.env.INTERVAL_MINUTES : 5} minutes`);
});

client.on("interactionCreate", async (interaction) => {
  if (!interaction.isChatInputCommand()) return;

  switch (interaction.commandName) {
    case "clear-forum":
      await handleClearForum(interaction, client);
      break;
  }
});

client.login(token);