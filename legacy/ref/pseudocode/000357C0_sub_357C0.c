/* ea:   0x000357C0 */
/* name: sub_357C0 */
/* size: 88 */
int __fastcall sub_357C0(_WORD *a1, unsigned __int16 a2)
{
  unsigned int v2; // r3
  int v4; // r1

  v2 = (unsigned __int16)*a1;
  if ( a2 < v2 )
  {
    v4 = (v2 - a2) << 16;
    *a1 = HIWORD(v4);
    return HIWORD(v4);
  }
  else
  {
    *a1 = 0;
    return 0;
  }
}

