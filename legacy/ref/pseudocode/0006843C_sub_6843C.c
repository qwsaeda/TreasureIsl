/* ea:   0x0006843C */
/* name: sub_6843C */
/* size: 88 */
int __fastcall sub_6843C(int *a1, int a2)
{
  sub_67370(a1, a2, nullptr);
  (*(void (__fastcall **)(_DWORD, int))(a2 + 36))(*(_DWORD *)(a2 + 40), a1[10]);
  (*(void (__fastcall **)(_DWORD, int))(a2 + 36))(*(_DWORD *)(a2 + 40), a1[9]);
  (*(void (__fastcall **)(_DWORD, int *))(a2 + 36))(*(_DWORD *)(a2 + 40), a1);
  return 0;
}

