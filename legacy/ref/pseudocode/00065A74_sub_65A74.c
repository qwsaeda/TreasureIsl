/* ea:   0x00065A74 */
/* name: sub_65A74 */
/* size: 64 */
char *__fastcall sub_65A74(int a1, char *a2)
{
  int v4; // r1

  v4 = *(_DWORD *)(a1 + 4) - (_DWORD)a2;
  if ( v4 >> 1 != 0 )
    memmove(a2, a2 + 2, 2 * (v4 >> 1));
  *(_DWORD *)(a1 + 4) -= 2;
  return a2;
}

