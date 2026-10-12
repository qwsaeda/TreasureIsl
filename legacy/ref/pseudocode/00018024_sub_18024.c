/* ea:   0x00018024 */
/* name: sub_18024 */
/* size: 112 */
int __fastcall sub_18024(_DWORD *a1)
{
  int v2; // r3
  unsigned int i; // r4

  v2 = *a1;
  for ( i = 0; i < (a1[1] - *a1) >> 1; v2 = *a1 )
  {
    *(_WORD *)(*a1 + 2 * i) = tolower(*(unsigned __int16 *)(v2 + 2 * i));
    ++i;
  }
  return *a1;
}

