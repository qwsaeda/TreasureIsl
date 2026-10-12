/* ea:   0x00055728 */
/* name: sub_55728 */
/* size: 92 */
int __fastcall sub_55728(int a1, int a2, __int16 a3, int a4, __int16 a5, __int16 a6)
{
  sub_466D0((_DWORD *)a1);
  *(_DWORD *)(a1 + 208) = a2;
  *(_WORD *)(a1 + 212) = a3;
  *(_DWORD *)a1 = &off_73990;
  *(_DWORD *)(a1 + 216) = a4;
  sub_57064(a1 + 136);
  sub_3053C((unsigned __int16 *)(a1 + 136), a5);
  sub_32410((unsigned __int16 *)(a1 + 136), a6);
  return a1;
}

