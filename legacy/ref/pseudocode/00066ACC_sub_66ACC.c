/* ea:   0x00066ACC */
/* name: sub_66ACC */
/* size: 56 */
char *__fastcall sub_66ACC(int a1, char *a2)
{
  char *v4; // r0
  char *v5; // r1

  v4 = *(char **)(a1 + 4);
  v5 = a2 + 4;
  if ( v4 != v5 )
    memmove(a2, v5, v4 - v5);
  *(_DWORD *)(a1 + 4) -= 4;
  return a2;
}

