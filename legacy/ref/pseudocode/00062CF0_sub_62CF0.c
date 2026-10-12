/* ea:   0x00062CF0 */
/* name: sub_62CF0 */
/* size: 120 */
int __fastcall sub_62CF0(HWND a1, int a2, int a3, int a4)
{
  _DWORD *v8; // r0
  int (__fastcall *v9)(HWND, int, int, int); // r5
  void *v10; // r0

  v8 = (_DWORD *)sub_62E68(&dword_811A8);
  v8[1] = a1;
  v9 = (int (__fastcall *)(HWND, int, int, int))(v8 + 2);
  v10 = (void *)(*(int (__fastcall **)(_DWORD *))(*v8 + 4))(v8);
  sub_62F00((int)v9, v10);
  SetWindowLongW(a1, -4, (LONG)v9);
  return v9(a1, a2, a3, a4);
}

