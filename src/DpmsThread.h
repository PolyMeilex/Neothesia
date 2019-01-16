#ifndef __DPMS_THREAD_H
#define __DPMS_THREAD_H

#include <thread>
#include <atomic>
#include <mutex>
#include <condition_variable>
#include <string>

#ifndef SCRIPTDIR
#define SCRIPTDIR "../scripts"
#endif

class DpmsThread
{
  std::atomic<bool>       m_is_keyboard_active{false};
  std::atomic<bool>       m_should_exit{false};
  std::condition_variable m_should_exit_cv;
  std::mutex              m_should_exit_cv_m;
  std::string             m_delay_screensaver_cmd;

  std::thread m_thread;

  void run()
  {
    while (!m_should_exit)
    {
      // Required for wait_for
      std::unique_lock<std::mutex> cv_lock(m_should_exit_cv_m);

      // Sleep for 5 seconds or until program exit
      // std::condition_variable::wait_for unlocks mutex
      if (std::cv_status::no_timeout ==
            m_should_exit_cv.wait_for(cv_lock, std::chrono::seconds(5)))
        // Handle exit (m_should_exit == true)
        continue;

      if (m_is_keyboard_active == false)
        //  no activity
        continue;

      // Handle timeout
      delayScreensaver();

      // Reset value
      m_is_keyboard_active.store(false);
    }
  }

  void delayScreensaver()
  {
    system(m_delay_screensaver_cmd.c_str());
  }

  public:
  DpmsThread() :
    m_thread(&DpmsThread::run, this),
    m_delay_screensaver_cmd(std::string(SCRIPTDIR) + "/delay_screensaver.sh")
  {
  }

  ~DpmsThread()
  {
    m_should_exit.store(true);
    m_should_exit_cv.notify_all();
    m_thread.join();
  }

  void handleKeyPress()
  {
    m_is_keyboard_active.store(true);
  }
};

#endif // __DPMS_THREAD_H
