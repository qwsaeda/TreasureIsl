/* ea:   0x00019630 */
/* name: sub_19630 */
/* size: 40 */
CFrameWnd *__fastcall sub_19630(CFrameWnd *a1, unsigned int a2, int a3)
{
  char v4; // r4
  CFrameWnd *result; // r0

  v4 = a2;
  CFrameWnd::OnSysCommand(a1, a2, a3);
  result = a1;
  if ( (v4 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

