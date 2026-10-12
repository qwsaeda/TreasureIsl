/* ea:   0x00019580 */
/* name: sub_19580 */
/* size: 36 */
void __fastcall sub_19580(CFrameWnd **a1, unsigned int a2, int a3)
{
  CFrameWnd *v3; // r0
  CFrameWnd *v4; // r4

  v3 = *a1;
  v4 = v3;
  if ( v3 != nullptr )
  {
    CFrameWnd::OnSysCommand(v3, a2, a3);
    operator delete(v4);
  }
}

