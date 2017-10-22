#ifndef MULTITOOL_HPP
#define MULTITOOL_HPP

#include "error.hpp"
#include "guiitem.hpp"
#include "virtualdrive.hpp"
#include <lvm2app.h>
#include <vector>

class Multitool : public GUIItem {
private:
  lvm_t m_lvm;
  vg_t m_volgroup;
  lv_t m_thinpool;
  std::vector<VirtualDrive> m_drives;
  std::vector<GUIEventHandler *> m_list_items;
  std::vector<GUIEventHandler *>::iterator m_selection;

  void update_list_items();
  bool has_selection() const;

  Multitool();
  Multitool(const Multitool &) = delete;
  Multitool &operator=(const Multitool &) = delete;

public:
  virtual ~Multitool();

  static Multitool &instance() {
    static Multitool multi;
    return multi;
  }

  std::vector<VirtualDrive> &drives() { return m_drives; }
  const std::vector<VirtualDrive> &drives() const { return m_drives; }
  const VirtualDrive &add_drive(uint64_t size);
  void remove_drive(const VirtualDrive &drive);
  void rescan_drives();

  float percent_used() const;

  virtual bool on_select() override;
  virtual bool on_next() override;
  virtual bool on_prev() override;

  virtual Bitmap render() const override;
};

#endif