/* ea:   0x0003C784 */
/* name: sub_3C784 */
/* size: 216 */
int __fastcall sub_3C784(int a1, int a2, int a3, int a4)
{
  int v6; // r5
  int v7; // r4
  int v8; // r4
  int v9; // r6
  int v10; // r0
  unsigned int v11; // r0

  v6 = sub_3AA5C(a3, a4);
  v7 = sub_16C9C((unsigned __int16 *)(v6 + 136), 4);
  v8 = *(unsigned __int8 *)(rand() % v7 + *(_DWORD *)(v6 + 164));
  v9 = sub_173C4(a4 + 652, v8);
  sub_16B48((_DWORD *)(v6 + 164), v8);
  if ( (_DWORD *)(a1 + 36) != dword_81184 )
    sub_17890((int)dword_81184, *(__int16 **)(a1 + 36), *(__int16 **)(a1 + 40));
  v10 = sub_12F04(aC);
  v11 = sub_3B0C0(dword_81184, (int)aC, 0, v10);
  return sub_3AF0C(dword_81184, v11, 2, v9 + 8);
}

