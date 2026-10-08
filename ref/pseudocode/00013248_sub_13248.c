/* ea:   0x00013248 */
/* name: sub_13248 */
/* size: 88 */
void *__fastcall sub_13248(int a1, void *a2, void *a3)
{
  int v6; // r2

  if ( a2 != a3 )
  {
    v6 = (*(_DWORD *)(a1 + 4) - (int)a3) >> 1;
    if ( v6 != -1 )
      memmove(a2, a3, 2 * (v6 + 1));
    *(_DWORD *)(a1 + 4) -= 2 * (((_BYTE *)a3 - (_BYTE *)a2) >> 1);
  }
  return a2;
}

