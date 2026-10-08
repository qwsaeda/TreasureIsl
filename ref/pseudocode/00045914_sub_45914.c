/* ea:   0x00045914 */
/* name: sub_45914 */
/* size: 72 */
_DWORD *__fastcall sub_45914(_DWORD *a1)
{
  sub_4595C();
  a1[12] = 0;
  a1[19] = DefWindowProcW;
  a1[18] = 0;
  *a1 = off_6FDA0;
  a1[11] = &off_6FD94;
  return a1;
}

