/* ea:   0x0003C8E8 */
/* name: sub_3C8E8 */
/* size: 176 */
void *__fastcall sub_3C8E8(int a1, int a2, int a3, _DWORD *a4)
{
  int v6; // r5
  int v7; // r0

  v6 = (unsigned __int8)(rand() % 9 + 1);
  v7 = rand();
  sub_54014(
    a4 + 152,
    v6,
    (unsigned __int8)(v7
                    - 21
                    * (((int)((unsigned __int64)(818089009LL * v7) >> 32) >> 2)
                     + ((int)((unsigned __int64)(818089009LL * v7) >> 32) >> 2 < 0))
                    + 1),
    1);
  if ( (_DWORD *)(a1 + 36) != dword_81184 )
    sub_17890((int)dword_81184, *(__int16 **)(a1 + 36), *(__int16 **)(a1 + 40));
  return sub_34B38(a4, (unsigned __int8)v6, 3);
}

