/* ea:   0x00063C88 */
/* name: sub_63C88 */
/* size: 96 */
void __fastcall sub_63C88(int a1)
{
  void *v2[7]; // [sp+0h] [bp-1Ch] BYREF

  (*(void (__fastcall **)(int))(*(_DWORD *)(a1 + 20) + 8))(a1 + 20);
  sub_64384(a1 + 8, *(_DWORD *)(a1 + 8), *(_DWORD *)(a1 + 12));
  memset(v2, 0, 12);
  sub_641A0(a1 + 8, v2);
  sub_1E81C(v2);
  *(_BYTE *)(a1 + 4) = 0;
}

