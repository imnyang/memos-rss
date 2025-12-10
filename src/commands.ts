import {
  SlashCommandBuilder,
  PermissionFlagsBits,
  ChatInputCommandInteraction,
  ForumChannel,
  Client,
} from "discord.js";

export const commands = [
  new SlashCommandBuilder()
    .setName("clear-forum")
    .setDescription("포럼 채널의 모든 게시물을 삭제합니다")
    .setDefaultMemberPermissions(PermissionFlagsBits.Administrator)
    .addChannelOption((option) =>
      option
        .setName("channel")
        .setDescription("삭제할 포럼 채널 (미지정 시 현재 채널)")
        .setRequired(false)
    )
    .toJSON(),
];

export async function handleClearForum(
  interaction: ChatInputCommandInteraction,
  client: Client
): Promise<void> {
  const targetChannel = interaction.options.getChannel("channel") || interaction.channel;

  if (!targetChannel) {
    await interaction.reply({ content: "Channel is not found", ephemeral: true });
    return;
  }

  const channel = await client.channels.fetch(targetChannel.id);

  if (!channel || !(channel instanceof ForumChannel)) {
    await interaction.reply({ content: "Channel is not a forum channel", ephemeral: true });
    return;
  }

  await interaction.deferReply({ ephemeral: true });

  try {
    const [activeThreads, archivedThreads] = await Promise.all([
      channel.threads.fetch(),
      channel.threads.fetchArchived(),
    ]);

    const allThreads = [
      ...activeThreads.threads.values(),
      ...archivedThreads.threads.values(),
    ];

    if (allThreads.length === 0) {
      await interaction.editReply("No posts to delete.");
      return;
    }

    let deleted = 0;
    let failed = 0;

    for (const thread of allThreads) {
      try {
        await thread.delete();
        deleted++;
      } catch (error) {
        console.error(`Failed to delete thread: ${thread.name}`, error);
        failed++;
      }
    }

    await interaction.editReply(
      `Deleted: ${deleted}\nFailed: ${failed}`
    );
  } catch (error) {
    console.error("Error occurred while deleting forum posts:", error);
    await interaction.editReply("❌ An error occurred while deleting posts.");
  }
}
