/* ea:   0x00062A14 */
/* name: sub_62A14 */
/* size: 272 */
int __fastcall sub_62A14(int a1)
{
  if ( IsWindow(*(HWND *)(a1 + 48)) || *(_BYTE *)(a1 + 36) != 0 && (unsigned __int8)sub_62C24(a1) != 0 )
    return 0;
  dword_7FBB0 = 3;
  dword_7FBCC = 0;
  dword_7FBD4 = *(_DWORD *)(a1 + 16);
  if ( sub_62C54(a1 + 44, 0, &unk_7D208, *(_DWORD *)(a1 + 4), 0x10000000, 0, 0, 0) == 0 )
    return 0;
  SHFullScreen(*(HWND *)(a1 + 48), 0x2Au);
  GXOpenInput();
  MoveWindow(*(HWND *)(a1 + 48), 0, 0, 240, 320, false);
  ShowWindow(*(HWND *)(a1 + 48), 5);
  UpdateWindow(*(HWND *)(a1 + 48));
  return (*(int (__fastcall **)(int))(*(_DWORD *)a1 + 20))(a1);
}

