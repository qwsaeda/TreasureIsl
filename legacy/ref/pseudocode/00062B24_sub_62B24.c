/* ea:   0x00062B24 */
/* name: sub_62B24 */
/* size: 96 */
void __fastcall sub_62B24(HWND *a1)
{
  if ( IsWindow(a1[12]) )
  {
    (*((void (__fastcall **)(HWND *))*a1 + 6))(a1);
    SHFullScreen(a1[12], 0x15u);
    GXCloseInput();
    DestroyWindow(a1[12]);
    PostQuitMessage(0);
  }
}

