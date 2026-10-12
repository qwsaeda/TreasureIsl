/* ea:   0x00019548 */
/* name: sub_19548 */
/* size: 56 */
void __fastcall sub_19548(CFrameWnd **a1, unsigned int a2, int a3)
{
  CFrameWnd *v5; // r4

  v5 = *a1;
  if ( (CFrameWnd *)a2 != *a1 && v5 != nullptr )
  {
    CFrameWnd::OnSysCommand(*a1, a2, a3);
    operator delete(v5);
  }
  *a1 = (CFrameWnd *)a2;
}

