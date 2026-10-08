/* ea:   0x0003C9B8 */
/* name: sub_3C9B8 */
/* size: 336 */
int __fastcall sub_3C9B8(int a1, int a2, int a3, int a4)
{
  int v7; // r7
  int v8; // r6
  unsigned __int16 v9; // r8
  int v10; // r5
  int v11; // r0
  unsigned int v12; // r0
  int v13; // r0
  unsigned int v14; // r0
  wchar_t v16[32]; // [sp+8h] [bp-40h] BYREF

  v7 = (unsigned __int8)(rand() % 21 + 1);
  v8 = sub_3AA5C(a3, a4);
  LOWORD(a3) = sub_16C9C((unsigned __int16 *)(v8 + 136), 3);
  v9 = sub_16C9C((unsigned __int16 *)(v8 + 136), 0) - a3;
  sub_30810((int *)(v8 + 152), v7, v9);
  v10 = sub_3B874(a4 + 608, v7);
  if ( (_DWORD *)(a1 + 36) != dword_81184 )
    sub_17890((int)dword_81184, *(__int16 **)(a1 + 36), *(__int16 **)(a1 + 40));
  v11 = sub_12F04(aG);
  v12 = sub_3B0C0(dword_81184, (int)aG, 0, v11);
  sub_3AF0C(dword_81184, v12, 2, v10 + 4);
  itow(v9, v16, 10);
  v13 = sub_12F04(aR);
  v14 = sub_3B0C0(dword_81184, (int)aR, 0, v13);
  return sub_3AFD8(dword_81184, v14, 2, v16);
}

