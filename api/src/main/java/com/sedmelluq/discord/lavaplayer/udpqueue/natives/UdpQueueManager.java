// Copyright belongs to https://github.com/sedmelluq/jda-nas
package com.sedmelluq.discord.lavaplayer.udpqueue.natives;

import com.sedmelluq.lava.common.natives.NativeResourceHolder;
import java.net.InetSocketAddress;
import java.nio.ByteBuffer;

/**
 * Manages sending out queues of UDP packets at a fixed interval.
 */
public class UdpQueueManager extends NativeResourceHolder {
  private final int bufferCapacity;
  private final ByteBuffer packetBuffer;
  private final UdpQueueManagerLibrary library;
  private final long instance;
  private boolean released;

  /**
   * @param bufferCapacity Maximum number of packets in one queue
   * @param packetInterval Time interval between packets in a queue
   * @param maximumPacketSize Maximum packet size
   */
  public UdpQueueManager(int bufferCapacity, long packetInterval, int maximumPacketSize) {
    this.bufferCapacity = bufferCapacity;
    packetBuffer = ByteBuffer.allocateDirect(maximumPacketSize);
    library = UdpQueueManagerLibrary.getInstance();
    instance = library.create(bufferCapacity, packetInterval);
  }

  /**
   * If the queue does not exist yet, returns the maximum number of packets in a queue.
   *
   * @param key Unique queue identifier
   * @return Number of empty packet slots in the specified queue
   */
  public int getRemainingCapacity(long key) {
    synchronized (library) {
      if (released) {
        return 0;
      }

      return library.getRemainingCapacity(instance, key);
    }
  }

  /**
   * @return Total capacity used for queues in this manager.
   */
  public int getCapacity() {
    return bufferCapacity;
  }

  /**
   * Adds one packet to the specified queue. Will fail if the maximum size of the queue is reached. There is no need to
   * manually create a queue, it is automatically created when the first packet is added to it and deleted when it
   * becomes empty.
   *
   * @param key Unique queue identifier
   * @param packet Packet to add to the queue
   * @param address The remote socket address
   * @return True if adding the packet to the queue succeeded
   */
  public boolean queuePacket(long key, ByteBuffer packet, InetSocketAddress address) {
    synchronized (library) {
      if (released) {
        return false;
      }

      int length = packet.remaining();
      packetBuffer.clear();
      packetBuffer.put(packet);

      int port = address.getPort();
      String hostAddress = address.getAddress().getHostAddress();
      return library.queuePacket(instance, key, hostAddress, port, packetBuffer, length);
    }
  }

  /**
   * Adds one packet to the specified queue that will be sent with the specified socket.
   * Will fail if the maximum size of the queue is reached. There is no need to manually create a queue,
   * it is automatically created when the first packet is added to it and deleted when it becomes empty.
   *
   * @param key Unique queue identifier
   * @param packet Packet to add to the queue
   * @param address The remote socket address
   * @param explicitSocket The socket file descriptor to use to send the packet
   * @return True if adding the packet to the queue succeeded
   */
  public boolean queuePacketWithSocket(long key, ByteBuffer packet, InetSocketAddress address, long explicitSocket) {
    synchronized (library) {
      if (released) {
        return false;
      }

      int length = packet.remaining();
      packetBuffer.clear();
      packetBuffer.put(packet);

      int port = address.getPort();
      String hostAddress = address.getAddress().getHostAddress();
      return library.queuePacketWithSocket(instance, key, hostAddress, port, packetBuffer, length, explicitSocket);
    }
  }

  /**
   * This is the method that should be called to start processing the queues. It will use the current thread and return
   * only when close() method is called on the queue manager.
   */
  public void process() {
    library.process(instance);
  }

  @Override
  protected void freeResources() {
    synchronized (library) {
      released = true;
      library.destroy(instance);
    }
  }

  /**
   * Simulate a GC pause stop-the-world by starting a heap iteration via JVMTI. The behaviour of this stop-the-world is
   * identical to that of an actual GC pause, so nothing in Java can execute during the pause.
   *
   * @param length Length of the pause in milliseconds
   */
  public static void pauseDemo(int length) {
    UdpQueueManagerLibrary.getInstance();
    UdpQueueManagerLibrary.pauseDemo(length);
  }
}
